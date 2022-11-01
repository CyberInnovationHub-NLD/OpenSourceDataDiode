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

use crate::errors::*;
use framework_constants::*;
use spsc_bip_buffer::BipBufferWriter;
use std::io::Read;
use std::net::Shutdown;
use std::os::unix::net::UnixStream;

pub struct BufferedSocketReader {
    stream: UnixStream,
    writer: BipBufferWriter,
}

impl BufferedSocketReader {
    ///Creates a new instance of BufferedSocketReader
    ///This function will block until the socket has been created by a SocketWriter.
    /// # Arguments
    /// * `path` - The path of the socket the reader should connect to.
    /// * `writer` - The BipBufferWriter used to send the received data to a bip_buffer.
    pub fn new(path: &str, writer: BipBufferWriter) -> Result<BufferedSocketReader> {
        //wait for socket to exist
        while !std::path::Path::new(path).exists() {
            std::thread::sleep(std::time::Duration::from_secs(2));
            log::warn!("BufferedSocketReader: socketfile does not yet exist.");
        }
        //wait for accept() to be called on socket
        loop {
            if let Ok(stream) = UnixStream::connect(path) {
                stream
                    .set_nonblocking(false)
                    .chain_err(|| "non blocking for BufferedSocketReader could not be set")?;
                stream
                    .set_write_timeout(None)
                    .chain_err(|| "write timeout for BufferedSocketReader could not be set!")?;
                return Ok(BufferedSocketReader { stream, writer });
            } else {
                std::thread::sleep(std::time::Duration::from_millis(200));
                log::warn!("BufferedSocketReader: accept has not yet been called on this socket");
            }
        }
    }

    ///This function fetches data from the socket.
    ///This data is then sent to the bip_buffer using the bipBufferWriter.
    ///This function will block until space is available in the bip_buffer.
    pub fn receive_data(&mut self) -> Result<usize> {
        //receive length field
        let mut exact_length_buffer = [0; BIP_BUFFER_LEN_FIELD_LEN];
        self.stream
            .read_exact(&mut exact_length_buffer)
            .chain_err(|| "Error reading exact when reading element length from stream")?;
        let element_length = usize::from_le_bytes(exact_length_buffer);

        //reserve total buffer space
        let mut reservation = loop {
            match self
                .writer
                .reserve(element_length + BIP_BUFFER_LEN_FIELD_LEN)
            {
                None => std::thread::sleep(std::time::Duration::from_millis(20)),
                Some(reservation) => break reservation,
            }
        };

        reservation[..BIP_BUFFER_LEN_FIELD_LEN].copy_from_slice(&exact_length_buffer);
        let element_reservation =
            &mut reservation[BIP_BUFFER_LEN_FIELD_LEN..element_length + BIP_BUFFER_LEN_FIELD_LEN];
        //receive data packet
        self.stream
            .read_exact(element_reservation)
            .chain_err(|| "Error reading exact when reading element from stream")?;
        Ok(element_length)
    }
    ///Stops the BufferedSocketReader. Calls Shutdown::Both on the underlying stream.
    pub fn stop(&self) -> Result<()> {
        log::warn!("Error shutting down socket for BufferedSocketReader");
        self.stream
            .shutdown(Shutdown::Both)
            .chain_err(|| "Error shutting down socket for BufferedSocketReader")?;
        log::info!("BufferedSocketReader has been shutdown");
        Ok(())
    }
}
