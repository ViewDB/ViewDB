// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
extern crate config;
extern crate viewdb_engine;
extern crate num_cpus;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate mio;

extern crate pumpkindb_mio_server as server;

use viewdb_engine::pumpkindb_engine::{script, storage, timestamp, lmdb};
use viewdb_engine::pumpkindb_engine::script::dispatcher;
use viewdb_engine::pumpkindb_engine::nvmem::MmapedFile;

use viewdb_engine::ViewDBDispatcher;

use clap::{App, Arg};

use std::thread;

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use mio::channel as mio_chan;

lazy_static! {
 static ref ARGS : clap::ArgMatches<'static> = {
   App::new("ViewDB Server")
        .version(crate_version!())
        .about("http://viewdb.org")
        .author("ViewDB Contributors")
        .setting(clap::AppSettings::ColoredHelp)
        .arg(Arg::with_name("config")
            .help("Configuration file")
            .required(false)
            .long("config")
            .short("c")
            .default_value("viewdb.toml")
            .takes_value(true))
        .get_matches()
 };
}
lazy_static! {
 static ref CONFIG : config::Config = {
    let mut config = config::Config::default()
        .merge(config::File::with_name(ARGS.value_of("config").unwrap()).required(false))
        .unwrap();
    config.set_default("server.port", 9981);
    config.set_default("storage.path", "viewdb.db");
    config
 };
}

lazy_static! {
 static ref ENVIRONMENT: lmdb::Environment = {
    let storage_path = CONFIG.get_str("storage.path").unwrap();
    fs::create_dir_all(storage_path.as_str()).expect("can't create directory");
    let map_size = CONFIG.get_int("storage.mapsize");
    let maxreaders = CONFIG.get_int("storage.maxreaders").and_then(|v| Ok(v as u32));
    if let Ok(max) = maxreaders {
       if max < 1 {
          error!("storage.maxreaders can't be less than 1");
          ::std::process::exit(1);
       }
    }
    storage::create_environment(storage_path, map_size.ok(), maxreaders.ok())
 };
}


pub fn main() {
    let storage_path = CONFIG.get_str("storage.path").unwrap();
    fs::create_dir_all(storage_path.as_str()).expect("can't create directory");

    let mut nvmem_pathbuf = PathBuf::from(storage_path);
    nvmem_pathbuf.push("nvmem");
    nvmem_pathbuf.set_extension("dat");
    let mut nvmem = MmapedFile::new(nvmem_pathbuf, 20).unwrap();
    let nvmem_hlc = nvmem.claim(20).unwrap();

    // Initialize logging
    let log_config = CONFIG.get_str("logging.config");
    let mut log_configured = false;
    if log_config.is_ok() {
        let log_file_path = log_config.unwrap();
        if fs::metadata(&log_file_path).is_ok() {
            log4rs::init_file(&log_file_path, Default::default()).unwrap();
            log_configured = true;
        } else {
            println!("{} not found", &log_file_path);
        }
    }

    if !log_configured {
        let appender = log4rs::config::Appender::builder()
            .build("console",
                   Box::new(log4rs::append::console::ConsoleAppender::builder().build()));
        let root =
            log4rs::config::Root::builder().appender("console").build(log::LogLevelFilter::Info);
        let _ = log4rs::init_config(log4rs::config::Config::builder()
            .appender(appender)
            .build(root)
            .unwrap());
        warn!("No logging configuration specified, switching to console logging");
    }

    info!("Starting up");

    let mut senders = Vec::new();

    let (relay_sender, relay_receiver) = mio_chan::channel();
    let mut client_messaging = viewdb_engine::pumpkindb_engine::messaging::Simple::new();
    let publisher_accessor = client_messaging.accessor();
    let subscriber_accessor = client_messaging.accessor();
    let _ = thread::spawn(move || client_messaging.run());
    let storage = Arc::new(storage::Storage::new(&ENVIRONMENT));
    let timestamp = Arc::new(timestamp::Timestamp::new(nvmem_hlc));

    let cpus = num_cpus::get();
    info!("Starting {} schedulers", cpus);
    for i in 0..cpus {
        debug!("Starting scheduler on core {}.", i);
        let (mut scheduler, sender) =
            script::Scheduler::new(
                ViewDBDispatcher::new(dispatcher::StandardDispatcher::new(storage.clone(),
                                                    publisher_accessor.clone(), subscriber_accessor.clone(),
                                                    timestamp.clone())));
        thread::spawn(move || scheduler.run());
        senders.push(sender)
    }

    server::run(CONFIG.get_int("server.port").unwrap(),
                senders, relay_sender, relay_receiver);
}