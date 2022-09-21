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
use bip_utils::write_to_bip_buffer;
use framework_constants::*;
use statistics_handler::*;
use transport_udp::rx::udp_receiver::*;
use transport_udp::tx::udp_sender::*;

#[test]
fn send_message() {
    let receiver_ip: &str = "0.0.0.0:9540";
    let sender_ip: &str = "0.0.0.0:9541";
    let receiver = UdpReceiver::new(receiver_ip).expect("Error creating receiver");
    let (receiver_writer, mut receiver_reader) =
        spsc_bip_buffer::bip_buffer_with_len(MAX_BIP_BUFFER_MESSAGE_SIZE * 10);

    //create statistics handler
    let statistics_client = StatsdClient::<StatsAllHandlers>::new_standard();
    let stats_data = statistics_client.data;

    let stats_data2 = stats_data.clone();

    std::thread::spawn(move || {
        receiver
            .run(receiver_writer, stats_data2)
            .expect("error while running receiver");
    });
    //send over udp
    let (mut sender_writer, sender_reader) =
        spsc_bip_buffer::bip_buffer_with_len(MAX_BIP_BUFFER_MESSAGE_SIZE * 10);
    let sender = UdpSender::new(sender_ip, sender_reader, 5, stats_data.clone())
        .expect("cant create udp sender");
    sender.run(receiver_ip).expect("error");

    //add data to the sender_bip_buffer
    let mut send_buffer = create_send_buffer();
    write_to_bip_buffer(&mut sender_writer, &mut send_buffer);
    //assert on data
    let mut receive_buffer = vec![0; MAX_BIP_BUFFER_MESSAGE_SIZE * 10];
    let message_size = read_from_bip_buffer(&mut receiver_reader, &mut receive_buffer);
    assert_ne!(message_size, 0);
    assert_eq!(
        &send_buffer[..message_size],
        &receive_buffer[..message_size]
    );
}

fn create_send_buffer() -> Vec<u8> {
    (0..=255).cycle().take(1_048_576).collect::<Vec<u8>>()
}
