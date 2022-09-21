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
use framework_constants::BIP_BUFFER_LEN_FIELD_LEN;
use framework_constants::MAX_BUFFER_SIZE_BYTES;
use framework_constants::MAX_PAYLOAD_SIZE_BYTES;
use logging::set_syslog;
use spsc_bip_buffer::bip_buffer_with_len;
use spsc_bip_buffer::BipBufferWriter;
use statistics_handler::*;
use std::thread::JoinHandle;
use structopt::*;
use transport_udp::tx::tx_arguments::OptSender;
use transport_udp::tx::udp_sender::UdpSender;

fn main() {
    let opt = OptSender::from_args();

    set_syslog(
        &opt.from_host_sys_log,
        &opt.from_port_sys_log.to_string(),
        &opt.to_host_sys_log,
        &opt.to_port_sys_log.to_string(),
        &opt.log_level,
        &opt.handler_name,
    )
    .expect("error setting syslog");
    let (writer, reader) =
        bip_buffer_with_len((MAX_BUFFER_SIZE_BYTES + BIP_BUFFER_LEN_FIELD_LEN) * 10); //stores 10 elements
    log::info!("Starting sender at {}:{}", opt.sender_addr, opt.sender_port);
    let statistics_client = StatsdClient::<StatsAllHandlers>::new_standard();
    let stats_data = statistics_client.data;
    let sender = UdpSender::new(
        &format!("{}:{}", opt.sender_addr, opt.sender_port),
        reader,
        opt.send_delay_ms,
        stats_data,
    )
    .expect("Error while setting udp sender");
    let test_data_handle = start_sending_test_data(writer);
    let sender_handle = sender
        .run(&format!("{}:{}", opt.receiver_addr, opt.receiver_port))
        .expect("Cant run transport udp");
    sender_handle.join().expect("Error joining sender thread");
    test_data_handle
        .join()
        .expect("Error joining mock data thread");
}

fn start_sending_test_data(mut writer: BipBufferWriter) -> JoinHandle<()> {
    std::thread::spawn(move || loop {
        send_single_large_message(&mut writer);
    })
}

fn _send_small_messages(writer: &mut BipBufferWriter) {
    let mut buffer = [0; 40];
    for (i, byte) in buffer.iter_mut().enumerate() {
        *byte = i as u8;
    }
    write_to_bip_buffer(writer, &buffer);
    write_to_bip_buffer(writer, &buffer[..20]);
    write_to_bip_buffer(writer, &buffer[..10]);
}

fn send_single_large_message(writer: &mut BipBufferWriter) {
    let mut buffer = vec![0; MAX_PAYLOAD_SIZE_BYTES * 16];
    for (i, byte) in buffer.iter_mut().enumerate() {
        *byte = i as u8;
    }
    write_to_bip_buffer(writer, &buffer);
}
