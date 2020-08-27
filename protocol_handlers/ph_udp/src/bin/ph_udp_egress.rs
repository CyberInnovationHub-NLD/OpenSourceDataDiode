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

use bip_utils::read_from_bip_buffer;
use logging::*;
use ph_udp::errors::*;
use ph_udp::*;
use socket_utils::buffered_socket_reader::BufferedSocketReader;
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

    outer_udp_egress().chain_unwrap();
}

fn outer_udp_egress() -> Result<()> {
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

    let (bip_writer, mut bip_reader) =
        bip_buffer_with_len(opt.bip_buffer_element_count * MAX_BIP_BUFFER_MESSAGE_SIZE);
    let mut socket_reader = BufferedSocketReader::new(&opt.socket_path, bip_writer)
        .chain_err(|| "Error while creating socket reader")?;

    //Start stats thread
    let stats_addr = format!("{}:{}", opt.host_stats_server, opt.port_stats_server);
    let stats: StatsdClient<StatsAllHandlers> =
        StatsdClient::<StatsAllHandlers>::new_with_custom_fields(None, None);
    stats
        .run(stats_addr, opt.handler_name)
        .chain_err(|| "Error while running statitics")?;

    let udp_receiver_server = format!("{}:{}", opt.udp_receiver_host, opt.udp_receiver_port);

    //2 threads:
    //- get_data_from_socket_send_to_bip_buffer
    //- udp_sender,

    // // {UNIX_DOMAIN_SOCKET} <-- get_data_from_socket_send_to_bip_buffer --> {BIPBUFFER} --> udp_sender --> {stats}

    let stats_data = stats.get_data_clone();
    //start get_data_from_socket_send_to_bip_buffer thread
    //Receive data from socket and send to bipbuffer
    let get_data_from_socket_send_to_bip_buffer = thread::Builder::new()
        .name("get_data_from_socket_send_to_bip_buffer".into())
        .spawn(move || loop {
            stats_data.in_bytes.add(
                socket_reader
                    .receive_data()
                    .chain_err(|| "Error in get_data_from_socket_send_to_bip_buffer thread")
                    .chain_unwrap() as u64,
            );
            stats_data.in_packets.add(1);
        })?;

    let socket = UdpSocket::bind(format!("0.0.0.0:{}", &opt.listening_port.to_string()))
        .chain_err(|| "error while parsing udp socket")?;

    let stats_data = stats.get_data_clone();
    //65535 is max udp pakket size
    let mut buffer = [0; MAX_UDP_SIZE];
    let stats_server: std::net::SocketAddr = udp_receiver_server
        .parse()
        .chain_err(|| "Cannot parse stats server and host to socket address")?;
    let udp_sender = thread::Builder::new()
        .name("udp_sender".into())
        .spawn(move || loop {
            let element_length = read_from_bip_buffer(&mut bip_reader, &mut buffer);
            match socket.send_to(&buffer[0..element_length], stats_server) {
                Ok(_) => {
                    stats_data.out_bytes.add(element_length as u64);
                    stats_data.out_packets.add(1);
                }
                Err(e) => {
                    stats_data.dropped_packets.add(1);
                    stats_data.dropped_bytes.add(element_length as u64);
                    log::warn!("Couldn't send udp packet to {}. Error {}:", stats_server, e)
                }
            }
        })?;

    //Joining threads

    get_data_from_socket_send_to_bip_buffer
        .join()
        .expect("Error joining get_data_from_socket_send_to_bip_buffer thread");
    udp_sender.join().expect("Error joining udp_sender thread");
    Ok(())
}
