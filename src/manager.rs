/*
 * watchmymcserver: a hacked together solution to start and stop minecraft servers.
 *
 * copyright (c) Eason Qin, 2024
 *
 * This source code form is licensed under the Mozilla Public License v2.0 and
 * comes AS-IS with NO WARRANTY
 */

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;
use std::{io, thread};

use chrono::{NaiveTime, Timelike};

use crate::config::Config;
use crate::server::MinecraftServer;

pub enum ServerManagerEvent {
    ServerOn,
    ServerOff,
    StopWatching,
    Nil,
}

pub struct MinecraftServerManager {
    server: Arc<Mutex<MinecraftServer>>,
    config: Config,
    timer_rx: Receiver<ServerManagerEvent>,
    timer_tx: Sender<ServerManagerEvent>,
}

impl MinecraftServerManager {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let server = MinecraftServer::new(config.clone())?;
        let (timer_tx, timer_rx): (Sender<ServerManagerEvent>, Receiver<ServerManagerEvent>) =
            mpsc::channel();

        //let timer_tx = Arc::new(timer_tx);
        //let t_timer_tx = timer_tx.clone();

        let res = MinecraftServerManager {
            server: Arc::new(Mutex::new(server)),
            config,
            timer_rx,
            timer_tx,
        };

        let on = chrono::NaiveTime::parse_from_str(&res.config.server.on, "%H:%M")?;
        let off = chrono::NaiveTime::parse_from_str(&res.config.server.off, "%H:%M")?;

        let t_timer_tx = res.timer_tx.clone();
        thread::spawn(move || loop {
            let now = chrono::Local::now().time();
            let tm_eq = |tm: &NaiveTime| now.minute() == tm.minute() && now.hour() == tm.hour();

            println!("NOW {:?} ON {:?} OFF {:?}", &now, &on, &off);

            if tm_eq(&on) {
                println!("sent on");
                t_timer_tx.send(ServerManagerEvent::ServerOn).unwrap();
            } else if tm_eq(&off) {
                println!("sent off");
                t_timer_tx.send(ServerManagerEvent::ServerOff).unwrap();
            }

            std::thread::sleep(Duration::new(5, 0));
        });

        Ok(res)
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        let ctrlc_tx = self.timer_tx.clone();
        ctrlc::set_handler(move || {
            ctrlc_tx.send(ServerManagerEvent::StopWatching).unwrap();
        })
        .expect("failed to set ctrl-c manager");

        loop {
            if let ServerManagerEvent::StopWatching = self.recv_loop()? {
                break;
            }
        }

        Ok(())
    }

    fn recv_loop(&mut self) -> anyhow::Result<ServerManagerEvent> {
        use ServerManagerEvent as E;

        let mut _handle: Option<JoinHandle<()>> = None;

        let evt = self.timer_rx.recv()?;

        match evt {
            E::ServerOn => {
                let started = { !self.server.lock().unwrap().started() };
                if started {
                    let t_server = self.server.clone();
                    _handle = Some(thread::spawn(move || {
                        t_server.lock().unwrap().start().unwrap();
                    }));
                }
            }
            E::ServerOff => {
                if let Some(h) = _handle {
                    self.server.lock().unwrap().stop().unwrap();
                    h.join().unwrap();
                    _handle = None;
                }
            }
            E::StopWatching => {
                return Ok(ServerManagerEvent::StopWatching);
            }
            E::Nil => {}
        }

        Ok(ServerManagerEvent::Nil)
    }
}
