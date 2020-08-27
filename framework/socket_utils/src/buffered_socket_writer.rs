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
use bip_utils::get_element_length;
use bip_utils::wait_for_data;
use framework_constants::BIP_BUFFER_LEN_FIELD_LEN;
use spsc_bip_buffer::BipBufferReader;
use std::io::Write;
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};

pub struct BufferedSocketWriter {
    stream: UnixStream,
    path: String,
}

impl BufferedSocketWriter {
    ///Creates a new instance of the SocketWriter and starts accepting connections to the socket.
    /// # Arguments
    /// * `path` - The path the socket is created on.
    pub fn start_listening(path: &str) -> Result<BufferedSocketWriter> {
        if let Err(e) = std::fs::remove_file(path) {
            log::error!("Error removing socket file at {}: {}", path, e);
        };
        Ok(BufferedSocketWriter {
            stream: create_stream(&path)?,
            path: path.to_string(),
        })
    }
    ///Used to send data to the socket. The data that is sent is read using `reader`.
    /// # Arguments
    /// * `reader` - The BipBufferReader used to get data from the bip_buffer.
    #[allow(clippy::unused_io_amount)]
    pub fn send_data(&mut self, reader: &mut BipBufferReader) -> Result<usize> {
        //read a usize from the buffer
        let element_length: usize = get_element_length(reader);
        //read data from the buffer
        wait_for_data(reader, element_length);
        let incoming = reader.valid();
        self.stream
            .write(&element_length.to_le_bytes())
            .chain_err(|| "Buffered Socket Writer could not send to socket")?;
        self.stream
            .write(&incoming[..element_length])
            .chain_err(|| "Buffered Socket Writer could not send to socket")?;
        reader.consume(element_length);
        Ok(element_length + BIP_BUFFER_LEN_FIELD_LEN)
    }

    pub fn stop(&self) {
        match self.stream.shutdown(Shutdown::Both) {
            Ok(_) => {
                log::info!("BufferedSocketWriter has been shutdown");
            }
            Err(e) => {
                log::warn!("{:?}", e);
            }
        }
        match std::fs::remove_file(&self.path) {
            Ok(_) => {
                log::info!("Cleanup succesfull. Shutdown complete.");
            }
            Err(e) => {
                log::warn!("Error while cleaning up: {}. Shutdown complete.", e);
            }
        };
    }
}

fn create_stream(path: &str) -> Result<UnixStream> {
    match UnixListener::bind(path) {
        Ok(listener) => match listener.accept() {
            Ok((stream, address)) => {
                log::info!("Client connected from: {:?}", address);
                stream
                    .set_nonblocking(false)
                    .chain_err(|| "non blocking for BufferedSocketWriter could not be set!")?;
                stream
                    .set_read_timeout(None)
                    .chain_err(|| "read timeout for BufferedSocketWriter could not be set!")?;
                Ok(stream)
            }
            Err(e) => Err(Error::with_chain(e, "Failed to accept incoming connection")),
        },
        Err(e) => Err(Error::with_chain(
            e,
            "Error while binding unix domain socket path",
        )),
    }
}
