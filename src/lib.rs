/*
 * watchmymcserver: a hacked together solution to start and stop minecraft servers.
 *
 * copyright (c) Eason Qin, 2024
 *
 * This source code form is licensed under the Mozilla Public License v2.0 and
 * comes AS-IS with NO WARRANTY
 */

use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, ErrorKind, Write},
    path::PathBuf,
};

use clap::Parser;
use colored::Colorize;
use duct::cmd;

macro_rules! io_err {
    ($err: literal) => {
        io::Error::new(ErrorKind::NotFound, $err)
    };
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    #[arg(help = "The server jarfile.")]
    jar: PathBuf,

    #[arg(short, long, help = "Where to put the server log.")]
    log: Option<PathBuf>,
    #[arg(short = 'I', long, help = "Where to put the standard input buffer.")]
    stdin: Option<PathBuf>,
    #[arg(
        short = 'Q',
        long,
        default_value_t = false,
        help = "Do not echo command ouput in stderr."
    )]
    quiet: bool,
    #[arg(short, long, help = "Java path.")]
    java: PathBuf,
    #[arg(
        short,
        long,
        help = "Amount of memory to allocate for the java heap in GIGABYTES, default 2.0"
    )]
    mem: Option<f32>,
}

pub struct MinecraftServer {
    java: PathBuf,
    jar: PathBuf,
    log: File,
    stdin: File,
    quiet: bool,
    mem: u16, // MB
}

impl MinecraftServer {
    pub fn new(
        java: PathBuf,
        jar: PathBuf,
        out_path: PathBuf,
        in_path: PathBuf,
        quiet: bool,
        mem: u16,
    ) -> Result<Self, io::Error> {
        let out_f = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(out_path)?;
        let in_f = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .read(true)
            .open(in_path)?;

        if !java.exists() {
            return Err(io_err!("java executable not found").into());
        }

        if !jar.exists() {
            return Err(io_err!("jarfile not found").into());
        }

        Ok(Self {
            java,
            jar,
            log: out_f,
            stdin: in_f,
            quiet,
            mem,
        })
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        let cmd = self.get_cmd();
        let reader = cmd
            .stderr_to_stdout()
            .stdout_capture()
            .stdin_file(self.stdin.try_clone().unwrap())
            .unchecked()
            .reader()?;
        let mut lines = BufReader::new(reader).lines();

        while let Some(Ok(line)) = lines.next() {
            let log_line = format!("[WATCHMYMCSERVER SERVER] {line}\n");
            if !self.quiet {
                print!("{log_line}");
            }
            self.log.write_all(log_line.as_bytes())?;
        }

        self.log
            .write_all(format!("[WATCHMYMCSERVER LOG] server exited",).as_bytes())?;

        Ok(())
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {
        self.stdin.write_all("stop".as_bytes())?;
        Ok(())
    }

    fn get_cmd(&self) -> duct::Expression {
        cmd!(
            self.java.to_str().unwrap(),
            "-jar",
            format!("-Xmx{}M", self.mem.to_string()),
            self.jar.to_str().unwrap(),
            "nogui"
        )
    }

    fn get_default_out_path() -> PathBuf {
        let path_s = "./mcserver_out.log";

        #[cfg(linux)]
        let path_s = "/var/log/watchmymcserver.log";

        return path_s.into();
    }

    fn get_default_in_path() -> PathBuf {
        let path_s = "./mcserver_stdin";

        #[cfg(linux)]
        let path_s = "/tmp/mcserver_stdin";

        return path_s.into();
    }
}

impl Into<MinecraftServer> for CliArgs {
    fn into(self) -> MinecraftServer {
        let out_path = self.log.unwrap_or(MinecraftServer::get_default_out_path());
        let in_path = self.stdin.unwrap_or(MinecraftServer::get_default_in_path());
        let mem = match self.mem {
            Some(mem) => (mem * 1000 as f32).floor() as u16,
            None => 2000,
        };

        MinecraftServer::new(self.java, self.jar, out_path, in_path, self.quiet, mem)
            .unwrap_or_else(|err| match err.kind() {
                ErrorKind::NotFound => panic!("{}not found:\n    {}", "error: ".bold().red(), err),
                ErrorKind::PermissionDenied => {
                    panic!("{}permission denied:\n    {}", "error".bold().red(), err)
                }
                _ => panic!("{}{}", "error: ".bold().red(), err),
            })
    }
}
