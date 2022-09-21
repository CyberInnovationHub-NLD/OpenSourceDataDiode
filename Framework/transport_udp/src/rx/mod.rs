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
use spsc_bip_buffer::BipBufferWriter;
use statistics_handler::StatsAllHandlers;
use std::net::UdpSocket;
use std::sync::Arc;

///This module contains the udp_receiver struct.
pub mod udp_receiver;

///This module contains the commandline arguments used by the udp_receiver.
pub mod rx_arguments;

///This module contains the inner_udp_receiver struct.
pub mod inner_udp_receiver;

///This function is used to receive a UDP packet on the given socket.
///The received data is placed in the supplied buffer.
/// # Arguments
/// * `socket` - The UdpSocket used to receive data on.
/// * `buffer` - The buffer used to store the received data.
/// # Returns
/// `usize` - The amount of bytes received.
pub fn receive_packet(socket: &UdpSocket, buffer: &mut [u8]) -> usize {
    match socket.recv_from(buffer) {
        Ok(received) => {
            log::debug!("Received packet with size {}", received.0);
            received.0
        }
        Err(e) => {
            //This can happen when the packet is lost before it reaches the rx side of the proxy.
            log::debug!("Couldn't receive packet: {}", e);
            0
        }
    }
}

///This function is used to read the information contained in the packet header of a received UDP packet.
///All packet information is placed inside a PacketData struct.
/// # Argument
/// * `buffer` - The packet to be read.
/// # Returns
/// `PacketData` - a struct containing all packet information read from the packet header.
pub fn read_packet_header(buffer: &[u8]) -> PacketData {
    let message_type = MessageType::from_u8(buffer[0]);
    let sequence_number_fixed: [u8; 4] = [buffer[1], buffer[2], buffer[3], buffer[4]];
    let sequence_number = u32::from_le_bytes(sequence_number_fixed);
    let payload_length_fixed: [u8; 2] = [buffer[5], buffer[6]];
    let payload_length = u16::from_le_bytes(payload_length_fixed);
    let remaining_messages_fixed: [u8; 2] = [buffer[7], buffer[8]];
    let remaining_messages = u16::from_le_bytes(remaining_messages_fixed);
    PacketData {
        message_type,
        sequence_number,
        payload_length,
        remaining_messages: remaining_messages as usize,
    }
}

///Packetloss is checked using the sequence number of the incoming packet.
///This sequence number should match the expected sequence number.
///If it doesn't packetloss has occured.
/// # Arguments
/// * `incoming` - The sequence number of the incoming packet.
/// * `current` - The currenctly known sequence number.
/// * `stats_data` - The struct used to store statistics data.
/// # Returns
/// `usize` - The amount of packets lost between incoming and current.
pub fn check_for_packetloss(
    incoming: u32,
    current: &mut u32,
    stats_data: Arc<StatsAllHandlers>,
) -> usize {
    let expected_sequence_number = *current + 1;
    let mut lost_packets: usize = 0;
    match incoming {
        //special case
        0 => {
            //do nothing
        }
        incoming if incoming == expected_sequence_number => {
            //do nothing, no packets were lost
        }
        //if packetloss has occurred
        incoming if incoming > expected_sequence_number => {
            let packetloss = incoming - expected_sequence_number;
            stats_data.packetloss.add(packetloss as u64);
            log::error!("Lost {} packets this iteration!", packetloss);
            lost_packets = packetloss as usize;
        }
        //if sequence number is somehow lower than expected
        _ => {
            log::warn!(
                "Packet with number: {} was received out of order!",
                incoming
            );
        }
    }
    *current = incoming;
    lost_packets
}

///This struct is used to store all header information of a UDP packet.
#[derive(Debug)]
pub struct PacketData {
    message_type: MessageType,
    sequence_number: u32,
    payload_length: u16,
    remaining_messages: usize,
}

#[cfg(test)]
mod test {
    use crate::rx::read_packet_header;
    use crate::tx::write_packet_header;
    use framework_constants::*;

    #[test]
    fn read_writer_packet_header_test() {
        let mut buffer = [0; MAX_BUFFER_SIZE_BYTES];
        let sequence_number = 12;
        let message_type = MessageType::Data.as_u8();
        let mut remaining_messages: u16 = 3;
        write_packet_header(
            &mut buffer,
            sequence_number,
            message_type,
            &mut remaining_messages,
        );
        let packet_header = read_packet_header(&buffer);
        assert_eq!(packet_header.message_type.as_u8(), message_type);
        assert_eq!(packet_header.payload_length, MAX_PAYLOAD_SIZE_BYTES as u16);
        assert_eq!(
            packet_header.remaining_messages,
            remaining_messages as usize
        );
    }

    #[test]
    #[ignore] //TODO: enable this again
    fn packetloss_test() {
        //TODO: create statistics struct

        // //check against 0
        // data.sequence_number_counter.store(0, Ordering::SeqCst);
        // data.packetloss_counter.store(0, Ordering::SeqCst);
        // check_for_packetloss(0, &mut 0, Arc::clone(&data));
        // assert_eq!(data.get_packetloss(), 0);

        // //check an expected increase in sequence number
        // data.sequence_number_counter.store(0, Ordering::SeqCst);
        // data.packetloss_counter.store(0, Ordering::SeqCst);
        // check_for_packetloss(2, &mut 0, Arc::clone(&data));
        // assert_eq!(data.get_packetloss(), 1);

        // //check for packetloss over single iteration
        // data.sequence_number_counter.store(0, Ordering::SeqCst);
        // data.packetloss_counter.store(0, Ordering::SeqCst);
        // check_for_packetloss(3, &mut 0, Arc::clone(&data));
        // assert_eq!(data.get_packetloss(), 2);

        // //check for packetloss over multiple iterations
        // data.sequence_number_counter.store(0, Ordering::SeqCst);
        // data.packetloss_counter.store(0, Ordering::SeqCst);
        // check_for_packetloss(1, &mut 0, Arc::clone(&data));
        // assert_eq!(data.get_packetloss(), 0);
        // data.sequence_number_counter.store(1, Ordering::SeqCst);
        // check_for_packetloss(4, &mut 1, Arc::clone(&data));
        // assert_eq!(data.get_packetloss(), 2);
        // data.sequence_number_counter.store(4, Ordering::SeqCst);
        // check_for_packetloss(20, &mut 4, Arc::clone(&data));
        // //expect 15 lost + 2 previously lost packets.
        // assert_eq!(data.get_packetloss(), 17);
    }
}
