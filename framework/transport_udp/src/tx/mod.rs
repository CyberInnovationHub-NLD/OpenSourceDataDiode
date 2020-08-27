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
use statistics_handler::*;
use std::net::UdpSocket;
use std::sync::Arc;

///The module containing the UdpSender struct.
pub mod udp_sender;

///The module containing the commandline arguments for UdpSender.
pub mod tx_arguments;

mod message_split;
mod special_message;

///This function is used to send the data contained in `buffer` using `socket`.
///When the packet is succesfully sent, the sequence number is incremented by one.
///Each time this function is called the sending of data is delayed by `send_delay_ms`.
fn send_data(
    socket: &UdpSocket,
    buffer: &mut [u8],
    sequence_number: &mut u32,
    send_delay_ms: u64,
    stats_data: Arc<StatsAllHandlers>,
) {
    *sequence_number = sequence_number.wrapping_add(1);
    std::thread::sleep(std::time::Duration::from_millis(send_delay_ms));
    match socket.send(buffer) {
        Ok(_) => {
            stats_data.out_bytes.add(buffer.len() as u64);
            stats_data.out_packets.add(1);
        }
        Err(e) => {
            log::warn!("{}", e);
        }
    }
}

///This function is used to write the packet header to a given buffer.
///This buffer should contain at least HEADER_SIZE_BYTES of empty space in front of the payload.
/// # Arguments
/// * `buffer` - The message buffer containing HEADER_SIZE_BYTES of free space at the front.
/// * `sequence_number` - The sequence number for this packet.
/// * `message_type` - The MessageType of this packet.
/// * `remaining_messages` - The amount of messages remaining when this packet is sent,
///should be 0 when this is the only message being sent.
pub fn write_packet_header(
    buffer: &mut [u8],
    sequence_number: u32,
    message_type: u8,
    remaining_messages: &mut u16,
) {
    if *remaining_messages != 0 {
        *remaining_messages -= 1;
    }
    debug_assert!(buffer.len() <= MAX_BUFFER_SIZE_BYTES);
    let sequence_number_bytes: [u8; 4] = sequence_number.to_le_bytes();
    let payload_length_bytes: [u8; 2] =
        ((buffer.len() as u16) - ((HEADER_SIZE_BYTES) as u16)).to_le_bytes();
    let remaining_messages: [u8; 2] = remaining_messages.to_le_bytes();
    buffer[0] = message_type;
    buffer[1] = sequence_number_bytes[0];
    buffer[2] = sequence_number_bytes[1];
    buffer[3] = sequence_number_bytes[2];
    buffer[4] = sequence_number_bytes[3];
    buffer[5] = payload_length_bytes[0];
    buffer[6] = payload_length_bytes[1];
    buffer[7] = remaining_messages[0];
    buffer[8] = remaining_messages[1];
}

#[cfg(test)]
mod test {
    use crate::tx::write_packet_header;
    use framework_constants::MessageType;
    use framework_constants::HEADER_SIZE_BYTES;
    use framework_constants::MAX_BUFFER_SIZE_BYTES;
    #[test]
    fn serialization_test() {
        let mut buffer: [u8; MAX_BUFFER_SIZE_BYTES] = [0; MAX_BUFFER_SIZE_BYTES];
        buffer[9] = 12;
        buffer[10] = 13;
        buffer[40] = 50;
        buffer[128] = 254;
        buffer[500] = 0;
        buffer[MAX_BUFFER_SIZE_BYTES - 1] = 81;
        let message_type = MessageType::Data;
        let sequence_number: u32 = 227;
        write_packet_header(
            &mut buffer,
            sequence_number,
            MessageType::as_u8(message_type),
            &mut 0,
        );
        //Check if serialization went as expected.
        //Check all edge cases
        let seq_bytes = 227u32.to_le_bytes();
        let len_bytes = ((MAX_BUFFER_SIZE_BYTES - HEADER_SIZE_BYTES) as u16).to_le_bytes();
        let remaining_messages = 0;
        //check message type
        assert_eq!(buffer[0], MessageType::Data.as_u8());
        //check sequence number
        assert_eq!(buffer[1], seq_bytes[0]);
        assert_eq!(buffer[2], seq_bytes[1]);
        assert_eq!(buffer[3], seq_bytes[2]);
        assert_eq!(buffer[4], seq_bytes[3]);
        //check payload length
        assert_eq!(buffer[5], len_bytes[0]);
        assert_eq!(buffer[6], len_bytes[1]);
        //check remaining messages
        assert_eq!(buffer[7], remaining_messages);
        assert_eq!(buffer[8], remaining_messages);
        //check edge cases inside payload
        assert_eq!(buffer[9], 12);
        assert_eq!(buffer[10], 13);
        assert_eq!(buffer[40], 50);
        assert_eq!(buffer[128], 254);
        assert_eq!(buffer[500], 0);
        assert_eq!(buffer[MAX_BUFFER_SIZE_BYTES - 1], 81);
    }
}
