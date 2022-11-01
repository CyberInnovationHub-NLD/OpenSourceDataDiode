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

use bincode::{deserialize, serialize_into, serialized_size};
//use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
/// Arguments for ph_kafka
pub mod arguments;
/// Implementation of Kafka consumer
pub mod consumer;
/// Error handling for ph_kafka with chain_error
pub mod errors;
/// Implementation of Kafka producer
pub mod producer;
use crate::errors::*;

/// The maximum size in bytes of a single bipbuffer message.
pub const MAX_BIP_BUFFER_MESSAGE_SIZE: usize = 1_050_000;

/// A struct for kafka message
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct KafkaMessage {
    /// The offset of the received message
    pub offset: i64,
    /// The message itself
    pub payload: Vec<u8>,
    /// The topic name of where the message came from
    pub topic: String,
}

impl KafkaMessage {
    pub fn new(payload: &[u8], offset: i64, topic: &str) -> KafkaMessage {
        KafkaMessage {
            offset,
            payload: payload.to_vec(),
            topic: topic.to_string(),
        }
    }
    pub fn serialize_packet(&self, buf: &mut [u8]) -> Result<u64> {
        match serialize_into(buf, self) {
            Ok(_) => match serialized_size(self) {
                Ok(v) => Ok(v),
                Err(e) => Err(Error::with_chain(e, "Failed serializing KafkaMessage")),
            },
            Err(e) => Err(Error::with_chain(e, "Failed serializing KafkaMessage")),
        }
    }

    pub fn deserialize_packet(buf: &[u8]) -> Result<KafkaMessage> {
        match deserialize(buf) {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::with_chain(e, "Failed deserializing KafkaMessage")),
        }
    }
}
