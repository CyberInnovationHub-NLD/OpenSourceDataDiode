// Copyright 2020 Ministerie van Defensie
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use logging::set_syslog;
use osdd::docker_runner::*;
use osdd::errors::Result;
use osdd::errors::*;
use osdd::read_toml::*;
use osdd::udp_multiplexer_stats::*;
use osdd::*;
use std::panic;
use std::thread;
use std::time;
use structopt::StructOpt;

const HANDLER_NAME_STRING: &str = "OSSD";

#[derive(StructOpt)]
struct Opt {
    #[structopt(short, long = "config_file", default_value = "/home/osdd/Config.toml")]
    config_file: String,
}

fn main() {
    panic::set_hook(Box::new(|panic_info| {
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            eprintln!("{}", s);
            log::error!("{}", s);
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            eprintln!("{}", s);
            log::error!("{}", s);
        } else {
            eprintln!(
                "No payload available in panic info! printing panic info: {}",
                panic_info
            );
            log::error!(
                "No payload available in panic info! printing panic info: {}",
                panic_info
            );
        }
        std::process::exit(1);
    }));

    osdd().chain_unwrap();
    loop {
        thread::sleep(time::Duration::from_millis(5000));
    }
}

/// This function starts the open source data diode.
/// It loads the configuration from a TOML file.
/// It starts a UDP multiplexer for statitics.
/// It create and execute commands to run Docker containers.
fn osdd() -> Result<()> {
    let opt = Opt::from_args();

    eprintln!("start {}", HANDLER_NAME_STRING);

    //Read handlers, settings_options and chains from the config file
    let toml_config = read_toml(&opt.config_file)?;

    let chain_handler_name = format!(
        "osdd.{}.{}",
        &toml_config.settings.instance, &toml_config.settings.network
    );

    //Syslog is created after all settings are read from the toml file
    set_syslog(
        &FROM_HOST_UDP_SYSLOG,
        &PORT_FROM_UDP_SYSLOG.to_string(),
        &toml_config.settings.syslog_host,
        &toml_config.settings.syslog_port,
        &toml_config.settings.log_level,
        &chain_handler_name,
    )
    .chain_err(|| "Error initializing syslog")?;

    //creating /sockets path
    match std::fs::create_dir(format!("{}/sockets", &toml_config.settings.path)) {
        Ok(_) => {
            log::trace!("created /sockets path");
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            log::trace!("/sockets already exist");
        }
        Err(e) => {
            return Err(Error::with_chain(
                e,
                format!(
                    "Error while creating {}/sockets path",
                    &toml_config.settings.path
                ),
            ))
        }
    }

    let stats_multiplexer_listening_port_u16 = toml_config
        .settings
        .stats_multiplexer_listening_port
        .parse::<u16>()?;

    //create commands to run processes
    let commands = create_commands_all_handlers(
        toml_config.chains,
        toml_config.handlers,
        stats_multiplexer_listening_port_u16,
        &toml_config.settings,
    )?;

    //start udp multiplexer in other thread
    run(
        stats_multiplexer_listening_port_u16,
        toml_config.settings.stats_servers,
    )?;

    //Starting dockers and monitoring
    handle_processes(commands)?;

    Ok(())
}
