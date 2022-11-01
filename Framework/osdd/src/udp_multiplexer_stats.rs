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
use crate::MAX_BUFFER_SIZE_BYTES;
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::thread::JoinHandle;

/// Receives all the stats form the handlers and send it to multiple addresses
pub fn run(stats_port: u16, stats_servers_string: Vec<String>) -> std::io::Result<JoinHandle<()>> {
    thread::Builder::new()
        .name("udp_multiplexer".into())
        .spawn(move || {
            mutiplex(stats_port, stats_servers_string).chain_unwrap();
        })
}

fn mutiplex(stats_port: u16, stats_servers_string: Vec<String>) -> Result<()> {
    let socket_addres = format!("0.0.0.0:{}", stats_port);
    let socket = UdpSocket::bind(socket_addres.to_string())
        .chain_err(|| format!("Binding udp socket on {}", socket_addres))?;

    let mut buffer = [0; MAX_BUFFER_SIZE_BYTES];
    let mut stats_servers: Vec<SocketAddr> = Vec::new();
    for stats_server_string in &stats_servers_string {
        stats_servers.push(stats_server_string.parse()?);
    }
    loop {
        match socket.recv_from(&mut buffer) {
            Ok((length, _)) => {
                log::trace!("Received packet with size {}", length);
                if length != 0 {
                    for stats_server in &stats_servers {
                        match socket.send_to(&buffer[0..length], stats_server) {
                            Ok(_) => (),
                            Err(e) => log::warn!(
                                "Couldn't send statsD message to {}. Error {}:",
                                stats_server,
                                e
                            ),
                        }
                    }
                }
            }
            Err(e) => {
                log::debug!("Couldn't receive statsd packet, error: {}", e);
            }
        }
    }
}
