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
use socket_utils::buffered_socket_writer::BufferedSocketWriter;
use spsc_bip_buffer::bip_buffer_with_len;
use statistics_handler::*;
use std::process;
use std::process::Command;
use std::sync::Arc;
use structopt::*;
use transport_udp::errors::ErrorKind::CommandError;
use transport_udp::errors::Result;
use transport_udp::errors::*;
use transport_udp::rx::rx_arguments::OptReceiver;
use transport_udp::rx::udp_receiver::UdpReceiver;

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

    clean_unwrap(udp_receive());
}

///The program will start multiple threads:
///* receiver_thread - The thread used by the UdpReceiver struct.
///* statistics_thread - The thread used by the StatisticsClient struct.
///* socket_writer_thread - The thread used to write data received by the UdpReceiver
///to a Unix Domain Socket.
fn udp_receive() -> Result<()> {
    let opt = OptReceiver::from_args();
    //Setup the logging to syslog for this application.
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

    //sets the niceness of the application.
    Command::new("renice")
        .args(&["-n", "-10", "-p", &process::id().to_string()])
        .spawn()
        .chain_err(|| CommandError("renice".to_string()))?;

    let (writer, mut reader) =
        bip_buffer_with_len(opt.bip_buffer_element_count * MAX_BIP_BUFFER_MESSAGE_SIZE);

    let receiver = Arc::new(UdpReceiver::new(&format!(
        "{}:{}",
        opt.receiver_addr, opt.receiver_port
    ))?);
    let stats_addr = format!("{}:{}", opt.host_stats_server, opt.port_stats_server);

    let statistics_client = StatsdClient::<StatsAllHandlers>::new_standard();
    statistics_client
        .run(stats_addr, opt.handler_name)
        .chain_err(|| "Error while running statitics")?;
    //build the udp_receiver thread.
    let receiver_thread_builder = std::thread::Builder::new().name("udp_receiver_thread".into());
    let receiver_handle = receiver_thread_builder.spawn(move || {
        let receiver = Arc::clone(&receiver);
        clean_unwrap(
            receiver
                .run(writer, statistics_client.data)
                .chain_err(|| "Error in thread udp_receiver"),
        )
    })?;

    let path = opt.socket_path.clone();
    //build the socket_writer thread.
    let socket_writer_thread_builder =
        std::thread::Builder::new().name("socket_writer_thread".into());
    let mut buffered_socket_writer = BufferedSocketWriter::start_listening(&path)
        .chain_err(|| "Error creating buffered socket writer")?;
    let socket_writer_handle = socket_writer_thread_builder.spawn(move || loop {
        clean_unwrap(
            buffered_socket_writer
                .send_data(&mut reader)
                .chain_err(|| "")
                .chain_err(|| "Error in socket_writer thread"),
        );
    })?;
    receiver_handle
        .join()
        .expect("Error joining receiver thread");
    socket_writer_handle
        .join()
        .expect("Error joining socket_writer thread");
    Ok(())
}
