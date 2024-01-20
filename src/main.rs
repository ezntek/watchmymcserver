/*
 * watchmymcserver: a hacked together solution to start and stop minecraft servers.
 *
 * copyright (c) Eason Qin, 2024
 *
 * This source code form is licensed under the Mozilla Public License v2.0 and
 * comes AS-IS with NO WARRANTY
 */

use std::sync::{Arc, Mutex};

use clap::Parser;
use watchmymcserver::{CliArgs, MinecraftServer};

fn main() -> anyhow::Result<()> {
    let server: MinecraftServer = CliArgs::parse().into();
    let server: Arc<Mutex<MinecraftServer>> = Arc::new(Mutex::new(server));

    let server_ctrlc = server.clone();
    ctrlc::set_handler(move || {
        println!("attempting to stop");
        let mut srv = server_ctrlc.lock().unwrap();
        srv.stop().unwrap();
    })?;

    {
        server.lock().unwrap().start().unwrap();
    }

    Ok(())
}
