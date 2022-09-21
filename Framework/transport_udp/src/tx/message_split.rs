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

use crate::tx::send_data;
use crate::tx::write_packet_header;
use bip_utils::get_element_length;
use bip_utils::wait_for_data;
use framework_constants::*;
use spsc_bip_buffer::BipBufferReader;
use statistics_handler::*;
use std::net::UdpSocket;
use std::sync::Arc;

///This function is used to split the data read from a bip_buffer.
///The data is split into packets that can be sent over UDP(payload < 65507 bytes)
pub fn split_and_send_data(
    socket: &UdpSocket,
    reader: &mut BipBufferReader,
    sequence_number: &mut u32,
    send_delay_ms: u64,
    stats_data: Arc<StatsAllHandlers>,
) {
    let element_length = get_element_length(reader);
    stats_data.in_bytes.add(element_length as u64);
    wait_for_data(reader, element_length);
    let element_buffer = &mut reader.valid()[..element_length];
    let mut remaining_messages: u16 = (element_length / MAX_PAYLOAD_SIZE_BYTES) as u16;
    let bytes_remaining = element_length % MAX_PAYLOAD_SIZE_BYTES;
    if bytes_remaining > 0 {
        remaining_messages += 1;
    }
    split_and_send_full_messages(
        socket,
        &mut remaining_messages,
        element_buffer,
        sequence_number,
        send_delay_ms,
        stats_data,
    );
    reader.consume(element_length);
}

fn split_and_send_full_messages(
    socket: &UdpSocket,
    remaining_messages: &mut u16,
    element_buffer: &mut [u8],
    sequence_number: &mut u32,
    send_delay_ms: u64,
    stats_data: Arc<StatsAllHandlers>,
) {
    let mut message_length_first_message = MAX_PAYLOAD_SIZE_BYTES;
    //check if message length is
    if message_length_first_message > element_buffer.len() {
        message_length_first_message = element_buffer.len();
    }
    let mut message_buffer = [0; MAX_BUFFER_SIZE_BYTES];
    let mut message = &mut element_buffer[0..message_length_first_message];
    message_buffer[HEADER_SIZE_BYTES..message_length_first_message + HEADER_SIZE_BYTES]
        .copy_from_slice(message);

    //create and send send first data message
    write_packet_header(
        &mut message_buffer[..message_length_first_message + HEADER_SIZE_BYTES],
        *sequence_number,
        MessageType::DataFirst.as_u8(),
        remaining_messages,
    );
    send_data(
        socket,
        &mut message_buffer[..message_length_first_message + HEADER_SIZE_BYTES],
        sequence_number,
        send_delay_ms,
        stats_data.clone(),
    );

    //create and send the rest of the messages
    for i in 1..=*remaining_messages as usize {
        let start_index = i * MAX_PAYLOAD_SIZE_BYTES;
        let mut end_index = start_index + MAX_PAYLOAD_SIZE_BYTES;
        if end_index > element_buffer.len() {
            end_index = element_buffer.len();
        }
        message = &mut element_buffer[start_index..end_index];
        message_buffer[HEADER_SIZE_BYTES..(end_index - start_index) + HEADER_SIZE_BYTES]
            .copy_from_slice(message);
        //send first message
        write_packet_header(
            &mut message_buffer[..(end_index - start_index) + HEADER_SIZE_BYTES],
            *sequence_number,
            MessageType::Data.as_u8(),
            remaining_messages,
        );
        send_data(
            socket,
            &mut message_buffer[..(end_index - start_index) + HEADER_SIZE_BYTES],
            sequence_number,
            send_delay_ms,
            stats_data.clone(),
        );
    }
}
