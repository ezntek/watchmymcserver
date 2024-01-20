/*
 * watchmymcserver: a hacked together solution to start and stop minecraft servers.
 *
 * copyright (c) Eason Qin, 2024
 *
 * This source code form is licensed under the Mozilla Public License v2.0 and
 * comes AS-IS with NO WARRANTY
 */

use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{io, thread};

use crate::config::Config;
use crate::server::MinecraftServer;

pub struct MinecraftServerManager {
    server: Arc<Mutex<MinecraftServer>>,
}

impl MinecraftServerManager {
    pub fn new(config: Config) -> Result<Self, io::Error> {
        let server = MinecraftServer::new(config.clone())?;
        Ok(MinecraftServerManager {
            server: Arc::new(Mutex::new(server)),
        })
    }

    pub fn start(&mut self) -> JoinHandle<()> {
        let ctrlc_server = self.server.clone();
        ctrlc::set_handler(move || {
            ctrlc_server.lock().unwrap().stop().unwrap();
        })
        .expect("failed to set ctrl-c manager");

        let thread_server = self.server.clone();
        return thread::spawn(move || {
            thread_server.lock().unwrap().start().unwrap();
        });
    }
}
