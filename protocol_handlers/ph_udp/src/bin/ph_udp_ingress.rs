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

use bip_utils::write_to_bip_buffer;
use error_chain::*;
use logging::*;
use ph_udp::errors::*;
use ph_udp::*;
use socket_utils::buffered_socket_writer::BufferedSocketWriter;
use spsc_bip_buffer::bip_buffer_with_len;
use statistics_handler::*;
use std::net::UdpSocket;
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

    outer_udp_ingress().chain_unwrap();
}

fn outer_udp_ingress() -> Result<()> {
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
    loop {
        match inner_udp_ingress() {
            Ok(_) => (),
            Err(e) => {
                log::error!("{}", e.display_chain());
            }
        }
        log::error!("Restarting ph_kafka_ingress");
        std::thread::sleep(std::time::Duration::from_millis(2000));
    }
}

///This handler is for udp packets
///in the ingress network it receive the udp packets and send it to the transport via a unix domain socket.
///in the egress network it receives the data from a unix domain socket and send it to a specific port.
fn inner_udp_ingress() -> Result<()> {
    let opt = arguments::OptIngress::from_args();

    let mut socket_writer = BufferedSocketWriter::start_listening(&opt.socket_path)
        .chain_err(|| "Error creating socket writer")?;

    let (mut bip_writer, mut bip_reader) =
        bip_buffer_with_len(MAX_BIP_BUFFER_MESSAGE_SIZE * opt.bip_buffer_element_count as usize);

    //Start stats thread
    let stats_addr = format!("{}:{}", opt.host_stats_server, opt.port_stats_server);
    let stats: StatsdClient<StatsAllHandlers> =
        StatsdClient::<StatsAllHandlers>::new_with_custom_fields(None, None);
    stats
        .run(stats_addr, opt.handler_name)
        .chain_err(|| "Error while running statitics")?;

    //2 threads:
    //- udp_receiver,
    //- bipreader_socketwriter

    // udp_receiver --> {BIPBUFFER_FIRST} <-- bipreader_socketwriter --> {UNIX_DOMAIN_SOCKET}

    let socket = UdpSocket::bind(format!("0.0.0.0:{}", &opt.listening_port.to_string()))?;

    let stats_data = stats.get_data_clone();

    let udp_receiver = thread::Builder::new()
        .name("udp_receiver".into())
        .spawn(move || {
            let mut buf = [0; MAX_UDP_SIZE];
            loop {
                match socket.recv_from(&mut buf) {
                    Ok((length, _)) => {
                        stats_data.in_packets.add(1);
                        stats_data.in_bytes.add(length as u64);
                        log::trace!("Received packet with size {}", length);
                        if length != 0 {
                            write_to_bip_buffer(&mut bip_writer, &buf[..length]);
                        }
                    }
                    Err(e) => {
                        log::debug!("Couldn't receive statsd packet, error: {}", e);
                    }
                }
            }
        })?;

    let bipreader_socketwriter = thread::Builder::new()
        .name("bipreader_socketwriter".into())
        .spawn(move || {
            let stats_data = stats.get_data_clone();
            loop {
                stats_data.out_bytes.add(
                    socket_writer
                        .send_data(&mut bip_reader)
                        .chain_err(|| "Error in bipreader_socketwriter thread")
                        .chain_unwrap() as u64,
                );
                stats_data.out_packets.add(1);
            }
        })?;

    //Joining threads
    udp_receiver
        .join()
        .expect("Error joining udp_receiver thread");
    bipreader_socketwriter
        .join()
        .expect("Error joining bipreader_socketwriter thread");

    Ok(())
}
