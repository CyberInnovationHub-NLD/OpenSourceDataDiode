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

use error_chain::ChainedError;
use logging::*;
use ph_kafka::consumer::serialize_between_bip_buffers;
use ph_kafka::consumer::IngressConsumer;
use ph_kafka::errors::Result;
use ph_kafka::errors::*;
use ph_kafka::*;
use socket_utils::buffered_socket_writer::*;
use spsc_bip_buffer::bip_buffer_with_len;
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

    outer_kafka_ingress().chain_unwrap();
}

fn outer_kafka_ingress() -> Result<()> {
    let opt = arguments::OptIngress::from_args();
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
            "echo '{} kafka-server-tx' >> /etc/hosts",
            opt.host_kafka_server
        )])
        .spawn()?;

    loop {
        match inner_kafka_ingress() {
            Ok(_) => (),
            Err(e) => {
                log::error!("{}", e.display_chain());
            }
        }
        log::error!("Restarting ph_kafka_ingress");
        std::thread::sleep(std::time::Duration::from_millis(2000));
    }
}

fn inner_kafka_ingress() -> Result<()> {
    let opt = arguments::OptIngress::from_args();
    let mut socket_writer = BufferedSocketWriter::start_listening(&opt.socket_path)
        .chain_err(|| "Error creating socket writer")?;

    let (mut bip_writer_first, mut bip_reader_first) =
        bip_buffer_with_len(MAX_BIP_BUFFER_MESSAGE_SIZE * opt.bip_buffer_element_count as usize);
    let (mut bip_writer_second, mut bip_reader_second) =
        bip_buffer_with_len(MAX_BIP_BUFFER_MESSAGE_SIZE * opt.bip_buffer_element_count as usize);

    //Start stats thread
    let stats_addr = format!("{}:{}", opt.host_stats_server, opt.port_stats_server);
    let stats: StatsdClient<StatsAllHandlers> =
        StatsdClient::<StatsAllHandlers>::new_with_custom_fields(None, Some("messages_behind"));
    stats
        .run(stats_addr, opt.handler_name)
        .chain_err(|| "Error while running statitics")?;

    //Create ingress_consumer
    let topicname = opt.topic_name;
    let mut ingress_consumer = IngressConsumer::new(
        &topicname,
        &opt.host_kafka_server,
        opt.port_kafka_server,
        opt.max_bytes_per_partition,
        stats.data.clone(),
    )?;

    //3 threads:
    //- kafka_poll_bipwriter,
    //- serialize_packet
    //- bipreader_socketwriter

    // {KAFKA-SERVER} <-- kafka_poll_bipwriter --> {BIPBUFFER_FIRST} <-- serialize_packet --> {BIPBUFFER_SECOND} <-- bipreader_socketwriter --> {UNIX_DOMAIN_SOCKET}

    //start bipreader_socketwriter thread
    let bipreader_socketwriter = thread::Builder::new()
        .name("bipreader_socketwriter".into())
        .spawn(move || loop {
            stats.data.clone().out_bytes.add(
                socket_writer
                    .send_data(&mut bip_reader_second)
                    .chain_err(|| "Error in bipreader_socketwriter thread")
                    .chain_unwrap() as u64,
            );
            stats.data.clone().out_packets.add(1);
        })?;

    //start serialize_packet thread
    let serialize_packet = thread::Builder::new()
        .name("serialize_packet".into())
        .spawn(move || loop {
            serialize_between_bip_buffers(
                &topicname,
                &mut bip_reader_first,
                &mut bip_writer_second,
            )
            .chain_err(|| "Error in thread serialize_between_bip_buffers")
            .chain_unwrap();
        })?;

    //Start kafka_poll_bipwriter
    let kafka_poll_bipwriter = thread::Builder::new()
        .name("kafka_poll_bipwriter".into())
        .spawn(move || {
            ingress_consumer
                .get_kafka_data_send_bip_buffer(&mut bip_writer_first)
                .chain_err(|| "Error in thread get_kafka_data_send_bip_buffer")
                .chain_unwrap();
        })?;

    kafka_poll_bipwriter
        .join()
        .expect("Error joining kafka_poll_bipwriter thread");
    bipreader_socketwriter
        .join()
        .expect("Error joining bipreader_socketwriter thread");
    serialize_packet
        .join()
        .expect("Error joining serialize_packet thread");

    Ok(())
}
