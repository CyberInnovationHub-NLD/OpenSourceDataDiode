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
use ph_kafka::errors::Result;
use ph_kafka::errors::*;
use ph_kafka::producer::EgressProducer;
use ph_kafka::*;
use socket_utils::buffered_socket_reader::*;
use spsc_bip_buffer::*;
use statistics_handler::*;
use std::process::Command;
use std::thread;
use structopt::StructOpt;

fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            log::error!("{}", s);
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            log::error!("{}", s);
        } else {
            log::error!(
                "No payload available in panic info! printing panic info: {}",
                panic_info
            );
        }
        std::process::exit(1);
    }));
    loop {
        kafka_egress().chain_unwrap();
        log::error!("Restarting ph_kafka_egress");
        std::thread::sleep(std::time::Duration::from_millis(2000));
    }
}

fn kafka_egress() -> Result<()> {
    let opt = arguments::OptEgress::from_args();
    set_syslog(
        &opt.from_host_sys_log,
        &opt.from_port_sys_log.to_string(),
        &opt.to_host_sys_log,
        &opt.to_port_sys_log.to_string(),
        &opt.log_level,
        &opt.handler_name,
    )
    .chain_err(|| "Error initializing syslog")?;
    log::info!("start {}", &opt.handler_name);

    //[OSDD-46]
    Command::new("/bin/sh")
        .args(&["-c"])
        .args(&[format!(
            "echo '{} kafka-server-rx' >> /etc/hosts",
            opt.host_kafka_server
        )])
        .spawn()?;

    let stats_addr = format!("{}:{}", opt.host_stats_server, opt.port_stats_server);
    let (bip_writer, mut bip_reader) =
        bip_buffer_with_len(opt.bip_buffer_element_count * MAX_BIP_BUFFER_MESSAGE_SIZE);
    let mut socket_reader = BufferedSocketReader::new(&opt.socket_path, bip_writer)
        .chain_err(|| "Error while create socket reader")?;

    //Start stats thread
    let stats: StatsdClient<StatsAllHandlers> = StatsdClient::<StatsAllHandlers>::new_standard();
    stats
        .run(stats_addr, opt.handler_name)
        .chain_err(|| "Error while running statitics")?;

    //Create EgressProducer
    let mut producer = EgressProducer::new(
        &opt.host_kafka_server,
        opt.port_kafka_server,
        opt.in_replacement,
        opt.out_replacement,
        stats.data,
    )?;

    // {UNIX_DOMAIN_SOCKET} <-- get_data_from_socket_send_to_bip_buffer --> {BIPBUFFER} <-- bipreader_send_to_kafka --> {KAFKA_SERVER}

    //start get_data_from_socket_send_to_bip_buffer thread
    //Receive data from socket and send to bipbuffer
    let get_data_from_socket_send_to_bip_buffer = thread::Builder::new()
        .name("get_data_from_socket_send_to_bip_buffer".into())
        .spawn(move || loop {
            socket_reader
                .receive_data()
                .chain_err(|| "Error in get_data_from_socket_send_to_bip_buffer thread")
                .chain_unwrap();
        })?;

    //read data from the bipbuffer and send it to a Kafka server
    let bipreader_send_to_kafka = thread::Builder::new()
        .name("bipreader_send_to_kafka".into())
        .spawn(move || {
            producer
                .get_data_from_bipbuffer_and_send_data_to_kafka(&mut bip_reader)
                .chain_err(|| "Error in thread bipreader_send_to_kafka")
                .chain_unwrap();
        })?;

    //join threads
    bipreader_send_to_kafka
        .join()
        .expect("Error joining bipreader_send_to_kafka thread");
    get_data_from_socket_send_to_bip_buffer
        .join()
        .expect("Error joining get_data_from_socket_send_to_bip_buffer");

    Ok(())
}
