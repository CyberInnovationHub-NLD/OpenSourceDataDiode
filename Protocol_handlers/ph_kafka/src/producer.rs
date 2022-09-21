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

use crate::errors::ErrorKind::*;
use crate::errors::*;
use crate::KafkaMessage;
use bip_utils::read_from_bip_buffer;
use kafka::producer::{Producer, Record};
use log::{info, warn};
use spsc_bip_buffer::BipBufferReader;
use statistics_handler::*;
use std::str;
use std::sync::Arc;

const BUFFER_SIZE_BYTES: usize = 1_050_000;

/// A struct with a Kafka producer and settings read form the command line arguments
pub struct EgressProducer {
    producer: Producer,
    stats_data: Arc<StatsAllHandlers>,
    in_replacement: String,
    out_replacement: String,
}

impl EgressProducer {
    pub fn new(
        host: &str,
        port: u16,
        in_replacement: String,
        out_replacement: String,
        stats_data: Arc<StatsAllHandlers>,
    ) -> Result<EgressProducer> {
        let host_port = format!("{}:{}", host, port);
        info!("Try to connect with kafka server: {}", host_port);
        let producer = Producer::from_hosts(vec![host_port]).create();

        match producer {
            Ok(producer) => Ok(EgressProducer {
                producer,
                stats_data,
                in_replacement,
                out_replacement,
            }),
            Err(e) => Err(Error::with_chain(e, "Failed creating Kafka producer")),
        }
    }

    ///Reads data from the bipbuffer and send it to kafka
    /// # Arguments
    /// * `bip_reader` - The BipBufferReader used to get data from the bip_buffer.
    pub fn get_data_from_bipbuffer_and_send_data_to_kafka(
        &mut self,
        mut bip_reader: &mut BipBufferReader,
    ) -> Result<()> {
        let mut buffer = [0; BUFFER_SIZE_BYTES];

        loop {
            let element_length = read_from_bip_buffer(&mut bip_reader, &mut buffer);
            self.deserialize_incoming_data_and_send_to_kafka(&buffer[..element_length])?;
        }
    }

    /// Give a u8 array, deserialize and send it to kafka
    /// # Arguments
    /// * `bip_reader` - The BipBufferReader used to get data from the bip_buffer.
    fn deserialize_incoming_data_and_send_to_kafka(&mut self, incoming_data: &[u8]) -> Result<()> {
        self.stats_data.in_bytes.add(incoming_data.len() as u64);
        self.stats_data.in_packets.add(1);

        if let Ok(kafka_message) = KafkaMessage::deserialize_packet(&incoming_data) {
            let topic = self.replace_topic(kafka_message.topic.to_string());
            let kafka_message_length = kafka_message.payload.len();
            match self
                .producer
                .send(&Record::from_value(&topic, kafka_message.payload))
            {
                Ok(_) => {
                    self.stats_data.out_bytes.add(kafka_message_length as u64);
                    self.stats_data.out_packets.add(1);
                }
                Err(e) => {
                    warn!("Error while sending data to kafka: {}", e);
                    self.stats_data.dropped_packets.add(1);
                    return Err(SendToKafka(e.to_string()).into());
                }
            }
        } else {
            warn!("invalid KafkaMessage received");
            self.stats_data.dropped_packets.add(1);
        }

        Ok(())
    }

    ///Replace the topic as it is given in command args
    /// # Arguments
    /// * `topic` - input topic.
    /// # Return
    /// * 'String' - Topic name, if was in give input list it is replaced.
    fn replace_topic(&self, mut topic: String) -> String {
        if self.in_replacement == topic {
            topic = self.out_replacement.to_string();
        }
        topic
    }
}
