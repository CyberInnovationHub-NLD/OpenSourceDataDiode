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


///The maximum size of the packet buffer.
///The field size sets a theoretical limit of 65,535 bytes (8 byte header + 65,527 bytes of data) for a UDP datagram.\
///However the actual limit for the data length, which is imposed by the underlying IPv4 protocol, is 65,507 bytes (65,535 − 8 byte UDP header − 20 byte IP header).
//src:https://stackoverflow.com/questions/1098897/what-is-the-largest-safe-udp-packet-size-on-the-internet
pub const MAX_BUFFER_SIZE_BYTES: usize = 65507;

///The size of the packet header.
//u8 + u32 + u16 + u16 = 9 bytes.
pub const HEADER_SIZE_BYTES: usize = 9;

///The maximum size in bytes the payload can use.
pub const MAX_PAYLOAD_SIZE_BYTES: usize = MAX_BUFFER_SIZE_BYTES - HEADER_SIZE_BYTES;

///The amount of times all special messages are sent.
pub const SPECIAL_MESSAGE_COUNT: usize = 200;

///The size in bytes of the length field used by the bip buffer.
pub const BIP_BUFFER_LEN_FIELD_LEN: usize = std::mem::size_of::<usize>();

///The maximum size in bytes of a single bipbuffer message.
//a bit more allocated then needed. 1_048_576(1 Mb) is needed + BIP_BUFFER_LEN_FIELD_LEN
pub const MAX_BIP_BUFFER_MESSAGE_SIZE: usize = 1_050_000;

///The messagetype used to determine the type of packet that was sent.
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum MessageType {
    StartUp = 1u8,
    HeartBeat = 2u8,
    DataFirst = 3u8,
    Data = 4u8,
    Shutdown = 5u8,
}

impl MessageType {
    ///Returns the byte value of a given MessageType.
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    ///Returns the MessageType for a given byte value.
    ///When no MessageType exists for the given byte value, MessageType::DataFirst is returned.
    /// # Arguments
    /// * `byte` - The byte value to get the MessageType of.
    ///
    /// *note: When no matching messagetype is found, the messagetype is set to DataFirst.*
    pub fn from_u8(byte: u8) -> MessageType {
        match byte {
            byte if byte == MessageType::as_u8(MessageType::StartUp) => MessageType::StartUp,
            byte if byte == MessageType::as_u8(MessageType::HeartBeat) => MessageType::HeartBeat,
            byte if byte == MessageType::as_u8(MessageType::DataFirst) => MessageType::DataFirst,
            byte if byte == MessageType::as_u8(MessageType::Data) => MessageType::Data,
            byte if byte == MessageType::as_u8(MessageType::Shutdown) => MessageType::Shutdown,
            _ => MessageType::DataFirst,
        }
    }
}
