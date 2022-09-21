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

use crate::errors::ErrorKind::UnixDomainSocketError;
use crate::errors::*;
use framework_constants::*;
use std::io::Read;
use std::net::Shutdown;
use std::os::unix::net::UnixStream;

pub struct SocketReader {
    stream: UnixStream,
}

impl SocketReader {
    ///Creates a new instance of SocketReader
    ///This function will block until the socket has been created by a SocketWriter.
    /// # Arguments
    /// * `path` - The path of the socket the reader should connect to.
    pub fn new(path: &str) -> Result<SocketReader> {
        //wait for socket to exist
        while !std::path::Path::new(path).exists() {
            std::thread::sleep(std::time::Duration::from_secs(2));
            log::warn!("SocketReader: socket at {} does not yet exist.", path);
        }
        //wait for accept() to be called on socket
        loop {
            if let Ok(stream) = UnixStream::connect(path) {
                stream
                    .set_nonblocking(false)
                    .chain_err(|| "non blocking for SocketReader could not be set!")?;
                stream
                    .set_write_timeout(None)
                    .chain_err(|| "write timeout for SocketReader could not be set!")?;
                return Ok(SocketReader { stream });
            } else {
                std::thread::sleep(std::time::Duration::from_millis(200));
                log::warn!("BufferedSocketReader: accept has not yet been called on this socket");
            }
        }
    }
    ///This function fetches data from the socket.
    ///This data is then copied to the supplied buffer.
    pub fn receive_data(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let mut element_length_bytes: [u8; BIP_BUFFER_LEN_FIELD_LEN] = Default::default();
        self.stream
            .read_exact(&mut element_length_bytes)
            .chain_err(|| "Error reading exact when reading element length from stream")?;
        let element_length = usize::from_le_bytes(element_length_bytes);
        if element_length > buffer.len() {
            return Err(UnixDomainSocketError(
                "Element length received by socket reader > buffer size".to_string(),
            )
            .into());
        }
        let element = &mut buffer[..element_length];
        //receive data packet
        self.stream
            .read_exact(element)
            .chain_err(|| "Error reading exact when reading element from stream")?;
        Ok(element_length)
    }
    ///Stops the SocketReader. Calls Shutdown::Both on the underlying stream.
    pub fn stop(&self) -> Result<()> {
        log::warn!("Error shutting down socket for SocketReader");
        self.stream.shutdown(Shutdown::Both)?;
        log::info!("SocketReader has been shutdown");
        Ok(())
    }
}
