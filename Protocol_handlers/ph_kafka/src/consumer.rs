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

use crate::errors::ErrorKind::SendToKafka;
use crate::errors::*;
use crate::MAX_BIP_BUFFER_MESSAGE_SIZE;
use bip_utils::read_from_bip_buffer;
use bip_utils::write_to_bip_buffer;
use error_chain::*;
use kafka::consumer::{Consumer, FetchOffset};
use log::trace;
use spsc_bip_buffer::BipBufferReader;
use spsc_bip_buffer::BipBufferWriter;
use statistics_handler::*;
use std::sync::Arc;

use std::str;

use crate::KafkaMessage;

const OFFSET_HEADER: usize = 8;

/// A struct with a Kafka consumer and settings read form the command line arguments
pub struct IngressConsumer {
    consumer: Consumer,
    topic: String,
    stats_data: Arc<StatsAllHandlers>,
    offset: i64,
}

impl IngressConsumer {
    pub fn new(
        topic: &str,
        host: &str,
        port: u16,
        max_bytes: usize,
        stats_data: Arc<StatsAllHandlers>,
    ) -> Result<IngressConsumer> {
        let host_port = format!("{}:{}", host, port);
        match Consumer::from_hosts(vec![host_port])
            .with_topic(topic.to_owned())
            //[OSSD-17]: Alle vs laatste berichten
            .with_fallback_offset(FetchOffset::Latest)
            //[OSDD-22]: Kafka consumer group configureerbaar
            .with_group("TestGroup2".to_owned())
            .with_fetch_max_bytes_per_partition(max_bytes as i32)
            .create()
        {
            Ok(consumer) => Ok(IngressConsumer {
                consumer,
                topic: topic.to_owned(),
                stats_data,
                offset: 0,
            }),
            Err(e) => Err(Error::with_chain(e, "Error while creating Kafka Consumer")),
        }
    }

    /// Get data from the kafka server and send it to bipbuffer
    /// Calculcates message behind after every poll
    /// # Arguments
    /// * `bip_writer` - The BipBufferWriter used to send data to the BipBuffer.
    pub fn get_kafka_data_send_bip_buffer(
        &mut self,
        bip_writer: &mut BipBufferWriter,
    ) -> Result<()> {
        let mut buf = [0; MAX_BIP_BUFFER_MESSAGE_SIZE];
        loop {
            self.poll_kafka(&mut buf, bip_writer)?;
            //[OSDD-21]: At least once/At most once/exaclty once configureerbaar
            self.consumer.commit_consumed()?;

            // Calculate message behind and store it in the statitics.
            let message_behind = match &self
                .consumer
                .client_mut()
                .fetch_topic_offsets(&self.topic, FetchOffset::Latest)
            {
                Ok(max_message) => max_message[0].offset - (self.offset + 1),
                Err(_) => -1,
            };
            self.stats_data
                .set_custom_gauge(message_behind as u64)
                .chain_err(|| "Erro whil setting message behind")?;
        }
    }

    /// Poll the kafka server and send it to bipbuffer
    /// Calculcates message behind after every poll
    /// # Arguments
    /// * `bip_writer` - The BipBufferWriter used to send data to the BipBuffer.
    fn poll_kafka(
        &mut self,
        buf: &mut [u8; MAX_BIP_BUFFER_MESSAGE_SIZE],
        bip_writer: &mut BipBufferWriter,
    ) -> Result<()> {
        match self.consumer.poll() {
            Ok(message_sets) => {
                for message_set in message_sets.iter() {
                    for message in message_set.messages() {
                        let message_length = message.value.len();
                        let message_offset = message.offset;

                        let offset_in_bytes: [u8; OFFSET_HEADER] = message_offset.to_be_bytes();
                        //fill first 8 bytes with the ofsset
                        buf[..OFFSET_HEADER].clone_from_slice(&offset_in_bytes[..OFFSET_HEADER]);
                        //fill other bytes with message
                        buf[OFFSET_HEADER..message_length + OFFSET_HEADER]
                            .clone_from_slice(message.value);
                        //send message to bipbuffer
                        write_to_bip_buffer(bip_writer, &buf[..message_length + OFFSET_HEADER]);

                        match self
                            .consumer
                            .consume_message(&self.topic, 0, message_offset)
                        {
                            Ok(_) => (),
                            Err(e) => trace!("error: {}", e),
                        }
                        self.offset = message_offset;
                        self.stats_data.in_bytes.add(message_length as u64);
                        self.stats_data.in_packets.add(1);
                    }
                }
                Ok(())
            }
            Err(e) => Err(SendToKafka(e.to_string()).into()),
        }
    }
}

///Read data from the bipbuffer, serialize_packet and send it to another bipbuffer
/// # Arguments
/// * `bip_reader` - The BipBufferWriter used to get data from the BipBuffer.
/// * `bip_writer` - The BipBufferWriter used to send data to the BipBuffer.
pub fn serialize_between_bip_buffers(
    topic: &str,
    bip_reader: &mut BipBufferReader,
    bip_writer: &mut BipBufferWriter,
) -> Result<()> {
    let mut buf: [u8; MAX_BIP_BUFFER_MESSAGE_SIZE] = [0; MAX_BIP_BUFFER_MESSAGE_SIZE];
    let length = read_from_bip_buffer(bip_reader, &mut buf);
    let mut offset_bytes: [u8; OFFSET_HEADER] = [0; OFFSET_HEADER];
    for byte in offset_bytes.iter_mut() {
        *byte = buf[*byte as usize];
    }
    let kafka_msg = KafkaMessage::new(
        &buf[OFFSET_HEADER..length],
        i64::from_be_bytes(offset_bytes),
        &topic,
    );

    match kafka_msg.serialize_packet(&mut buf) {
        Ok(length) => write_to_bip_buffer(bip_writer, &buf[..length as usize]),
        Err(e) => bail!(e),
    };
    Ok(())
}
