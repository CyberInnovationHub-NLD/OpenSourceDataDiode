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

use framework_constants::*;
use logging::set_syslog;
use socket_utils::buffered_socket_reader::BufferedSocketReader;
use spsc_bip_buffer::bip_buffer_with_len;
use statistics_handler::*;
use std::process;
use std::process::Command;
use std::thread::Builder;
use structopt::*;
use transport_udp::errors::ErrorKind::*;
use transport_udp::errors::Result;
use transport_udp::errors::*;
use transport_udp::tx::tx_arguments::OptSender;
use transport_udp::tx::udp_sender::UdpSender;

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

    clean_unwrap(udp_send());
}

///The program will start multiple threads:
///* sender_thread - The thread used by the UdpSender struct.
///* statistics_thread - The thread used by the StatisticsClient struct.
///* socket_reader_thread - The thread used to read data received from a protocol_handler.
///to a Unix Domain Socket.
fn udp_send() -> Result<()> {
    let opt = OptSender::from_args();
    set_syslog(
        &opt.from_host_sys_log.to_string(),
        &opt.from_port_sys_log.to_string(),
        &opt.to_host_sys_log.to_string(),
        &opt.to_port_sys_log.to_string(),
        &opt.log_level.to_string(),
        &opt.handler_name,
    )
    .chain_err(|| "Error initializing syslog")?;
    opt.log_config_info();
    let (writer, reader) =
        bip_buffer_with_len(opt.bip_buffer_element_count * MAX_BIP_BUFFER_MESSAGE_SIZE);

    //create statistics client
    let stats_addr = format!("{}:{}", opt.host_stats_server, opt.port_stats_server);
    let statistics_client = StatsdClient::<StatsAllHandlers>::new_standard();
    statistics_client
        .run(stats_addr, opt.handler_name)
        .chain_err(|| "Error while running statitics")?;
    let stats_data = statistics_client.data;

    let sender = UdpSender::new(
        &format!("{}:{}", opt.sender_addr, opt.sender_port),
        reader,
        opt.send_delay_ms,
        stats_data,
    )?;
    let mut unix_socket_reader: BufferedSocketReader =
        BufferedSocketReader::new(&opt.socket_path, writer)
            .chain_err(|| "Error creating buffered socket reader")?;
    Command::new("renice")
        .args(&["-n", "-10", "-p", &process::id().to_string()])
        .spawn()
        .chain_err(|| CommandError("renice".to_string()))?;

    let sender_handle = sender.run(&format!("{}:{}", opt.receiver_addr, opt.receiver_port))?;
    let unix_socket_thread_builder = Builder::new().name("socket_reader_thread".into());
    let unix_socket_handle = unix_socket_thread_builder
        .spawn(move || loop {
            clean_unwrap(
                unix_socket_reader
                    .receive_data()
                    .chain_err(|| "Error in socket reader thread"),
            );
        })
        .expect("Error spawning socket_reader_thread");
    sender_handle.join().expect("Error joining sender thread");
    unix_socket_handle
        .join()
        .expect("Error joining unix socket reader thread");
    Ok(())
}
