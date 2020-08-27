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
use ph_kafka::KafkaMessage;
use spsc_bip_buffer::BipBufferWriter;
use std::sync::Arc;

use std::str;

/// Input arguments for the filter handler
pub mod arguments;
pub mod errors;

///The maximum size in bytes of a single bipbuffer message.
//a bit more allocated then needed. 1_048_576(1 Mb) is needed + BIP_BUFFER_LEN_FIELD_LEN
pub const BUFFER_SIZE_BYTES: usize = 1_050_000;

///Check for the first bytes of a kafka message. If it matches the word_to_filter then it drops the data. Else it is written to the bipbuffer.
pub fn filtering(
    buffer: &[u8; BUFFER_SIZE_BYTES],
    element_length: usize,
    mut bip_writer_second: &mut BipBufferWriter,
    word_to_filter: &str,
    stats_data: &Arc<statistics_handler::StatsAllHandlers>,
) {
    match KafkaMessage::deserialize_packet(&buffer[..element_length]) {
        Ok(kafka_message) => {
            let message = kafka_message.payload;
            if message.len() >= word_to_filter.len() {
                match str::from_utf8(&message[0..word_to_filter.len()]) {
                    Ok(first_x_bytes_tex) => {
                        if first_x_bytes_tex == word_to_filter {
                            log::info!("Did not sent packet because it was filtered. Packet contained: {}!", word_to_filter);
                            stats_data.dropped_packets.add(1);
                            stats_data.dropped_bytes.add(element_length as u64);
                        } else {
                            write_to_bip_buffer(&mut bip_writer_second, &buffer[..element_length]);
                        }
                    }
                    //The the first x bytes of the incoming data is not utf8 then it cannot be checked and will be sent to the bipbuffer
                    Err(_) => {
                        write_to_bip_buffer(&mut bip_writer_second, &buffer[..element_length])
                    }
                }
            //The data is smaller then the word_to_filter it will never match so it will be sent to the bipbuffer
            } else {
                write_to_bip_buffer(&mut bip_writer_second, &buffer[..element_length])
            }
        }
        //The data cannot be converted to a kafka message. It is problably corrupted data so it will be dropped.
        Err(e) => {
            log::warn!("Cannot read the kafka message: {}", e);
            stats_data.dropped_packets.add(1);
            stats_data.dropped_bytes.add(element_length as u64);
        }
    };
}
