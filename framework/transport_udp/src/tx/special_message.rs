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

use crate::tx::write_packet_header;
use framework_constants::MessageType;
use framework_constants::HEADER_SIZE_BYTES;
use framework_constants::SPECIAL_MESSAGE_COUNT;
use std::net::UdpSocket;

///This function is used to send packets with MessageType::StartUp to the UdpReceiver.
pub fn send_startup_messages(socket: &UdpSocket, sequence_number: &mut u32) {
    log::info!("Started sending startup signals to receiver.");
    let mut buf = [0; HEADER_SIZE_BYTES];
    *sequence_number = 0;
    for _ in 0..SPECIAL_MESSAGE_COUNT {
        write_packet_header(&mut buf, 0, MessageType::StartUp.as_u8(), &mut 0);
        if let Err(e) = socket.send(&buf) {
            log::warn!("Failed sending startup message: {}", e);
        }
    }
}

///This function is used to send packets with MessageType::ShutDown to the UdpReceiver.
pub fn send_shutdown_messages(socket: &UdpSocket) {
    log::info!("Started sending shutdown signals to receiver.");
    let mut buf = [0; HEADER_SIZE_BYTES];
    for _ in 0..SPECIAL_MESSAGE_COUNT {
        write_packet_header(&mut buf, 0, MessageType::Shutdown.as_u8(), &mut 0);
        if let Err(e) = socket.send(&buf) {
            log::warn!("Failed sending shutdown message: {}", e);
        }
    }
}

///This function is used to send packets with MessageType::HeartBeat to the UdpReceiver.
fn _send_heartbeat_message(socket: &UdpSocket, sequence_number: u32) {
    log::info!("Heartbeat message has been sent.");
    let mut buf = [0; HEADER_SIZE_BYTES];
    write_packet_header(
        &mut buf,
        sequence_number,
        MessageType::Shutdown.as_u8(),
        &mut 0,
    );
    if let Err(e) = socket.send(&buf) {
        log::warn!("Failed sending heartbeat message: {}", e);
    }
}
