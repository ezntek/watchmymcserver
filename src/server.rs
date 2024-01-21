/*
 * watchmymcserver: a hacked together solution to start and stop minecraft servers.
 *
 * copyright (c) Eason Qin, 2024
 *
 * This source code form is licensed under the Mozilla Public License v2.0 and
 * comes AS-IS with NO WARRANTY
 */

use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, ErrorKind, Write};

use crate::config::Config;

macro_rules! io_err {
    ($err: literal) => {
        io::Error::new(ErrorKind::NotFound, $err)
    };
}

macro_rules! txt_log {
    ($text: literal) => {
        format!(
            "[WATCHMYMCSERVER LOG {}] {}\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            $text
        )
    };
}

pub struct MinecraftServer {
    config: Config,
    log_file: File,
    stdin_file: File,
    started: bool,
}

impl MinecraftServer {
    pub fn new(config: Config) -> Result<Self, io::Error> {
        let out_f = File::options()
            .write(true)
            .append(true)
            .create(true)
            .open(&config.server.log)?;
        let in_f = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .read(true)
            .open(&config.server.stdin)?;

        if !config.java.exists() {
            return Err(io_err!("java executable not found").into());
        }

        if !config.server.jar.exists() {
            return Err(io_err!("jarfile not found").into());
        }

        Ok(Self {
            config,
            started: false,
            log_file: out_f,
            stdin_file: in_f,
        })
    }

    pub fn start(&mut self) -> io::Result<()> {
        self.started = true;

        {
            let txt = txt_log!("starting server");
            self.log_file.write_all(txt.as_bytes())?;
            if !self.config.quiet {
                print!("{txt}");
            }
        }

        let cmd = self.get_cmd();
        let reader = cmd
            .stderr_to_stdout()
            .stdout_capture()
            .stdin_file(self.stdin_file.try_clone().unwrap())
            .unchecked()
            .reader()?;
        let mut lines = BufReader::new(reader).lines();

        while let Some(Ok(line)) = lines.next() {
            let log_line = format!(
                "[WATCHMYMCSERVER SERVER {}] {line}\n",
                chrono::Local::now().format("%Y-%m-%d")
            );
            if !self.config.quiet {
                print!("{log_line}");
            }
            self.log_file.write_all(log_line.as_bytes())?;
        }

        {
            let txt = txt_log!("server exited");
            self.log_file.write_all(txt.as_bytes())?;
            if !self.config.quiet {
                print!("{txt}");
            }
        }

        Ok(())
    }

    pub fn stop(&mut self) -> io::Result<()> {
        self.stdin_file.write_all("stop".as_bytes())?;
        self.started = false;

        Ok(())
    }

    pub fn started(&self) -> bool {
        self.started
    }

    fn get_cmd(&self) -> duct::Expression {
        let java = self.config.java.to_str().unwrap();
        let jar = self.config.server.jar.to_str().unwrap();
        let xmx = format!("-Xmx{}M", self.config.server.max_heap_size);
        let xms = format!("-Xms{}M", self.config.server.max_heap_size / 4);
        let soft_xmx = format!(
            "-XX:SoftMaxHeapSize={}M",
            self.config.server.soft_max_heap_size
        );

        let mut arglist = vec!["-jar", xmx.as_str(), soft_xmx.as_str(), xms.as_str()];
        for item in &self.config.server.extra_args {
            arglist.push(item);
        }
        arglist.push(jar);
        arglist.push("nogui");

        duct::cmd(java, arglist)
    }
}
