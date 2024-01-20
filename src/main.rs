/*
 * watchmymcserver: a hacked together solution to start and stop minecraft servers.
 *
 * copyright (c) Eason Qin, 2024
 *
 * This source code form is licensed under the Mozilla Public License v2.0 and
 * comes AS-IS with NO WARRANTY
 */

use std::path::PathBuf;

use clap::Parser;
use watchmymcserver::{config::Config, *};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    #[arg(short, long)]
    config: Option<PathBuf>,
}

fn main() {
    let args = CliArgs::parse();
    let config = Config::load(args.config).unwrap();
    let mut mgr = MinecraftServerManager::new(config).unwrap();
    mgr.start().join().unwrap();
}
