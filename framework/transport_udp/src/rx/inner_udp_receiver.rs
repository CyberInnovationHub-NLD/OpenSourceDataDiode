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

use crate::rx::*;
use bip_utils::write_to_bip_buffer;
use statistics_handler::StatsAllHandlers;
use std::net::UdpSocket;
use std::sync::Arc;
use MessageType::*;
use State::*;

///This enum is used by the state machine in InnerUdpReceiver.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum State {
    ///The WaitingForFirstData state is used when a message with the DataFirst MessageType is expected.
    WaitingForFirstData,
    ///The WaitingForData state is used when a message with the Data MessageType is expected.
    ///This state is given a usize that contains the total amount of messages that need to be combined.
    WaitingForData(usize),
}

///The InnerUdpReceiver is used by the UdpReceiver. It contains the real logic used in the UdpReceiver.
///The InnerUdpReceiver is a state machine. It will always start in the WaitingForFirstData state.
pub struct InnerUdpReceiver {
    socket: UdpSocket,
    bip_writer: BipBufferWriter,
    packet_buffer: Vec<u8>,
    combined_buffer: Vec<Vec<u8>>,
    current_sequence_number: u32,
    state: State,
    stats_data: Arc<StatsAllHandlers>,
}

impl InnerUdpReceiver {
    ///creates a new InnerUdpReceiver struct.
    /// # Arguments
    /// * `socket` - The udpSocket, used to receive data on.
    /// * `bip_writer` - The BipBufferWriter used to write combined data to the bip_buffer.
    /// * `stats_data` - The struct used to store statistics data.
    /// # Returns
    /// `InnerUdpReceiver`
    pub fn new(
        socket: UdpSocket,
        bip_writer: BipBufferWriter,
        stats_data: Arc<StatsAllHandlers>,
    ) -> InnerUdpReceiver {
        //create message buffer to store the received messages.
        let combined_buffer: Vec<Vec<u8>> = vec![Vec::with_capacity(MAX_PAYLOAD_SIZE_BYTES); 20];
        let packet_buffer = vec![0; MAX_BUFFER_SIZE_BYTES];
        let current_sequence_number = 0;
        InnerUdpReceiver {
            socket,
            bip_writer,
            packet_buffer,
            combined_buffer,
            current_sequence_number,
            state: State::WaitingForFirstData,
            stats_data,
        }
    }

    ///This function starts the InnerUdpReceiver state machine.
    ///This function will run on a seperate thread.
    /// # Returns
    /// `JoinHandle<()>` - The JoinHandle of the started thread.
    pub fn run(mut self) {
        loop {
            receive_packet(&self.socket, &mut self.packet_buffer);
            let packet_header = read_packet_header(&self.packet_buffer);
            let lost_packets = check_for_packetloss(
                packet_header.sequence_number,
                &mut self.current_sequence_number,
                self.stats_data.clone(),
            );
            self.update_in_stats(&packet_header);
            //When packetloss occurs the current data can not be used anymore.
            //State is set to WaitingForFirstData
            if lost_packets > 0 {
                self.state = WaitingForFirstData;
                log::warn!(
                    "Packetloss detected, {} packets lost. State set to WaitingForFirstData",
                    lost_packets
                );
            }
            if !self.update_state(&packet_header) {
                break;
            }
        }
    }

    ///This function is used to change the state depending MessageType of the incoming packet.
    fn update_state(&mut self, packet_header: &PacketData) -> bool {
        self.state = match (self.state, packet_header.message_type) {
            //Data when it is expected
            (WaitingForData(total), Data) => self.handle_data_message(&packet_header, total),

            //Data when it is not expected
            (_, Data) => {
                log::trace!("Data message discarded");
                WaitingForFirstData
            }

            //DataFirst when it is expected
            (WaitingForFirstData, DataFirst) => self.handle_data_first_message(&packet_header),

            //DataFirst when it is not expected
            (_, DataFirst) => {
                log::warn!(
                    "Received datafirst message when it was not expected, 
                    data messages were discarded"
                );
                self.handle_data_first_message(&packet_header)
            }

            //Startup always sets sequence number to 0
            (_, StartUp) => self.handle_startup_message(),

            //Heartbeat received, return previous state and log heartbeat
            (_, HeartBeat) => self.handle_heartbeat_message(),

            //Shutdown received, break the loop and stop the application
            (_, MessageType::Shutdown) => {
                self.handle_shutdown_message();
                return false;
            }
        };
        true
    }

    ///This function is used to handle a message that has the DataFirst MessageType.
    fn handle_data_first_message(&mut self, packet_header: &PacketData) -> State {
        if packet_header.remaining_messages > 0 {
            self.combined_buffer[0].clear();
            self.combined_buffer[0].extend_from_slice(
                &self.packet_buffer
                    [HEADER_SIZE_BYTES..packet_header.payload_length as usize + HEADER_SIZE_BYTES],
            );
            //the first count of remaining messages + 1 = total amount of messages
            WaitingForData(packet_header.remaining_messages + 1)
        } else {
            //datafirst is the only message
            write_to_bip_buffer(
                &mut self.bip_writer,
                &self.packet_buffer
                    [HEADER_SIZE_BYTES..packet_header.payload_length as usize + HEADER_SIZE_BYTES],
            );
            //update bytes out statistic
            self.stats_data
                .out_bytes
                .add((packet_header.payload_length as usize + BIP_BUFFER_LEN_FIELD_LEN) as u64);
            WaitingForFirstData
        }
    }

    ///This function is used to handle a message that has the Data MessageType.
    fn handle_data_message(&mut self, packet_header: &PacketData, total_messages: usize) -> State {
        //copy the data from the message into the combined buffer.
        let combined_buffer_position =
            &mut self.combined_buffer[total_messages - packet_header.remaining_messages - 1];
        combined_buffer_position.clear();
        combined_buffer_position.extend_from_slice(
            &self.packet_buffer
                [HEADER_SIZE_BYTES..packet_header.payload_length as usize + HEADER_SIZE_BYTES],
        );
        if packet_header.remaining_messages == 0 {
            //data element in combined_buffer is complete
            self.combine_and_write_to_bip(packet_header, total_messages);
            return State::WaitingForFirstData;
        }
        State::WaitingForData(total_messages)
    }

    ///This function is used to handle a message that has the Heartbeat MessageType.
    fn handle_heartbeat_message(&self) -> State {
        log::info!("Heartbeat message received");
        self.state
    }

    ///This function is used to handle a message that has the StartUp MessageType.
    fn handle_startup_message(&mut self) -> State {
        if self.current_sequence_number != 0 {
            self.current_sequence_number = 0;
            log::info!("Startup message has been received, sequence number was reset to 0");
        }
        State::WaitingForFirstData
    }

    ///This function is used to handle a message that has the ShutDown MessageType.
    fn handle_shutdown_message(&self) {
        log::warn!("Shutdown message received, breaking loop!");
    }

    ///This function is used to combine all packets that belong to one set of data.
    ///The combined messages are written to the BipBuffer.
    fn combine_and_write_to_bip(&mut self, packet_header: &PacketData, total_messages: usize) {
        let total_bytes = ((total_messages - 1) * MAX_PAYLOAD_SIZE_BYTES)
            + packet_header.payload_length as usize
            + BIP_BUFFER_LEN_FIELD_LEN;
        if let Some(mut reservation) = self.bip_writer.reserve(total_bytes) {
            //write length field
            let element_bytes = (total_bytes - BIP_BUFFER_LEN_FIELD_LEN).to_le_bytes();
            reservation[..BIP_BUFFER_LEN_FIELD_LEN].copy_from_slice(&element_bytes);
            //write parts
            let mut start_index = BIP_BUFFER_LEN_FIELD_LEN;
            let mut end_index = MAX_PAYLOAD_SIZE_BYTES + start_index;
            for message in 0..total_messages - 1 {
                reservation[start_index..end_index].copy_from_slice(&self.combined_buffer[message]);
                start_index = end_index;
                end_index += MAX_PAYLOAD_SIZE_BYTES;
            }
            //write last, possibly < MAX_PAYLOAD_SIZE_BYTE, part.
            reservation[start_index..].copy_from_slice(
                &self.combined_buffer[total_messages - 1][..packet_header.payload_length as usize],
            );

            //update bytes out statistic
            self.stats_data.out_bytes.add(total_bytes as u64);
            reservation.send();
        } else {
            self.stats_data.dropped_bytes.add(total_bytes as u64);
            log::warn!("Data dropped when writing to bip_buffer in receiver: No space in buffer!");
        }
    }

    ///This function will update the in_bytes and in_packets counter of the statistics struct.
    fn update_in_stats(&self, packet_header: &PacketData) {
        self.stats_data.in_packets.add(1);
        self.stats_data
            .in_bytes
            .add(packet_header.payload_length as u64);
    }
}

#[cfg(test)]
mod test {
    mod update_state {
        use crate::rx::inner_udp_receiver::*;
        use statistics_handler::*;
        use std::net::SocketAddr;
        use std::net::UdpSocket;
        #[test]
        fn update_state_1_message_test() {
            //create all needed parameters for inner_receiver
            let socket = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0))) //any free port
                .expect("Error binding port for test");
            let statistics_client = StatsdClient::<StatsAllHandlers>::new_standard();
            let stats_data = statistics_client.data;
            let (writer, _) = spsc_bip_buffer::bip_buffer_with_len(MAX_BIP_BUFFER_MESSAGE_SIZE);
            let packet_header = PacketData {
                message_type: MessageType::Data,
                payload_length: 0,
                remaining_messages: 0,
                sequence_number: 0,
            };

            //check initial state
            let mut inner_receiver = InnerUdpReceiver::new(socket, writer, stats_data);
            assert_eq!(inner_receiver.state, State::WaitingForFirstData);
            //update and check state for first message
            inner_receiver.update_state(&packet_header);
            assert_eq!(inner_receiver.state, State::WaitingForFirstData);
        }

        #[test]
        fn update_state_2_messages_test() {
            //create all needed parameters for inner_receiver
            let socket = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0))) //any free port
                .expect("Error binding port for test");
            let statistics_client = StatsdClient::<StatsAllHandlers>::new_standard();
            let stats_data = statistics_client.data;
            let (writer, _) =
                spsc_bip_buffer::bip_buffer_with_len(MAX_BIP_BUFFER_MESSAGE_SIZE * 10);
            let mut packet_header = PacketData {
                message_type: MessageType::DataFirst,
                payload_length: MAX_PAYLOAD_SIZE_BYTES as u16,
                remaining_messages: 1,
                sequence_number: 0,
            };

            //check initial state
            let mut inner_receiver = InnerUdpReceiver::new(socket, writer, stats_data);
            assert_eq!(inner_receiver.state, State::WaitingForFirstData);
            //update state for first message
            inner_receiver.update_state(&packet_header);
            assert_eq!(inner_receiver.state, State::WaitingForData(2));
            //update packet header and state
            packet_header.remaining_messages = 0;
            packet_header.sequence_number = 1;
            packet_header.message_type = MessageType::Data;
            inner_receiver.update_state(&packet_header);

            //check final state
            assert_eq!(inner_receiver.state, State::WaitingForFirstData);
        }

        #[test]
        fn update_state_16_messages_test() {
            //create all needed parameters for inner_receiver
            let socket = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0))) //any free port
                .expect("Error binding port for test");
            let statistics_client = StatsdClient::<StatsAllHandlers>::new_standard();
            let stats_data = statistics_client.data;
            let (writer, _) =
                spsc_bip_buffer::bip_buffer_with_len(MAX_BIP_BUFFER_MESSAGE_SIZE * 10);
            let mut packet_header = PacketData {
                message_type: MessageType::DataFirst,
                payload_length: MAX_PAYLOAD_SIZE_BYTES as u16,
                remaining_messages: 15,
                sequence_number: 0,
            };

            //check initial state
            let mut inner_receiver = InnerUdpReceiver::new(socket, writer, stats_data);
            assert_eq!(inner_receiver.state, State::WaitingForFirstData);
            //update state for first message
            inner_receiver.update_state(&packet_header);
            assert_eq!(inner_receiver.state, State::WaitingForData(16));

            //update packet header and state for other messages
            for i in 0..14 {
                packet_header.remaining_messages -= 1;
                packet_header.message_type = MessageType::Data;
                packet_header.sequence_number = i as u32 + 1;
                inner_receiver.update_state(&packet_header);
                assert_eq!(inner_receiver.state, State::WaitingForData(16));
            }

            //check state and update packet header for final packet
            packet_header.remaining_messages -= 1;
            packet_header.sequence_number = 15;
            packet_header.message_type = MessageType::Data;
            packet_header.payload_length = 10;
            inner_receiver.update_state(&packet_header);
            assert_eq!(inner_receiver.state, State::WaitingForFirstData);
        }
    }
}
