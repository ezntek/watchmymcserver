/*
 * watchmymcserver: a hacked together solution to start and stop minecraft servers.
 *
 * copyright (c) Eason Qin, 2024
 *
 * This source code form is licensed under the Mozilla Public License v2.0 and
 * comes AS-IS with NO WARRANTY
 */

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

static DEFAULT_CONFIG: &'static str = r#"
#![enable(implicit_some)]
Config(
    java: "/usr/local/bin/java", // path to your java executable
    quiet: false, 

    server: Server(
        base: "/var/minecraft/server",             // directory to run the JAR in
        jar: "/var/minecraft/softwares/paper.jar", // server JAR file
        on: "8:00",                                // when to turn the server on (24h time)
        off: "21:06",                              // when to turn the server off (24h time)
        log: "/var/log/wmms.log",                  // log
        stdin: "/tmp/wmms_stdin",                  // stdin buffer (don't change if you don't know
                                                   // what you're doing)
    
        // extra jvm opts
        max_heap_size: 4000,                                        // megabytes
        soft_max_heap_size: 3000,                                   // megabytes
        extra_args: ["-XX:+UnlockExperimentalVMOptions", "-XX:+UseZGC"], // replace with "" if using Java 13 or before
    ),
)
"#;

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub java: PathBuf,
    pub quiet: bool,

    pub server: Server,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Server {
    pub base: PathBuf,
    pub jar: PathBuf,
    pub log: PathBuf,
    pub stdin: PathBuf,
    pub on: String,
    pub off: String,

    pub max_heap_size: u32,
    pub soft_max_heap_size: u32,
    pub extra_args: Vec<String>,
}

impl Config {
    pub fn load(supplied_default: Option<PathBuf>) -> anyhow::Result<Config> {
        let path = match supplied_default {
            Some(p) => p,
            None => format!("{}/.watchmymcserver.ron", std::env::var("HOME").unwrap()).into(),
        };

        if !path.exists() {
            let mut f = File::options()
                .read(true)
                .write(true)
                .create(true)
                .open(&path)?;
            f.write_all(DEFAULT_CONFIG.as_bytes())?;
            Ok(ron::from_str::<Config>(DEFAULT_CONFIG)?)
        } else {
            let s = std::fs::read_to_string(&path)?;
            Ok(ron::from_str::<Config>(&s)?)
        }
    }
}
