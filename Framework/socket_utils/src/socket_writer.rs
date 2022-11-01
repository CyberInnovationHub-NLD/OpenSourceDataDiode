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
use std::io::Write;
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};

pub struct SocketWriter {
    stream: UnixStream,
    path: String,
}

impl SocketWriter {
    ///Creates a new instance of the SocketWriter
    /// # Arguments
    /// * `path` - The path the socket is created on.
    pub fn start_listening(path: &str) -> Result<SocketWriter> {
        if let Err(e) = std::fs::remove_file(path) {
            log::warn!("Error removing socket file: {}", e);
        };
        Ok(SocketWriter {
            stream: match UnixListener::bind(path) {
                Ok(listener) => match listener.accept() {
                    Ok((stream, address)) => {
                        log::info!("Client connected from: {:?}", address);
                        stream.set_nonblocking(false).chain_err(|| {
                            "non blocking for BufferedSocketWriter could not be set!"
                        })?;
                        stream.set_read_timeout(None).chain_err(|| {
                            "read timeout for BufferedSocketWriter could not be set!"
                        })?;
                        stream
                    }
                    Err(e) => {
                        return Err(Error::with_chain(e, "Failed to accept incoming connection"))
                    }
                },
                Err(e) => {
                    return Err(Error::with_chain(
                        e,
                        "Error while binding unix domain socket path",
                    ))
                }
            },
            path: path.to_string(),
        })
    }

    ///Sends data from `buffer` to the socket.
    #[allow(clippy::unused_io_amount)]
    pub fn send_data(&mut self, buffer: &mut [u8]) -> Result<()> {
        let mut stream_buffer = vec![0; buffer.len() + BIP_BUFFER_LEN_FIELD_LEN];

        let byte_len = buffer.len().to_le_bytes();
        stream_buffer[..BIP_BUFFER_LEN_FIELD_LEN].copy_from_slice(&byte_len);
        stream_buffer[BIP_BUFFER_LEN_FIELD_LEN..buffer.len() + BIP_BUFFER_LEN_FIELD_LEN]
            .copy_from_slice(&buffer);
        self.stream
            .write(&stream_buffer)
            .chain_err(|| "Socket writer could not write to socket")?;
        Ok(())
    }

    ///Stops the SocketWriter. Calls Shutdown::Both on the underlying stream.
    ///After the stream is stopped, the socket file is removed.
    pub fn stop(&self) {
        match self.stream.shutdown(Shutdown::Both) {
            Ok(_) => {
                log::info!("SocketWriter has been shutdown");
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
