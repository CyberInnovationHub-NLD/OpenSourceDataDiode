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

use crate::errors::Result;
use crate::rx::inner_udp_receiver::InnerUdpReceiver;
use crate::rx::*;
use statistics_handler::StatsAllHandlers;
use std::net::UdpSocket;
use std::sync::Arc;
pub struct UdpReceiver {
    socket: UdpSocket,
}

impl UdpReceiver {
    pub fn new(host: &str) -> Result<UdpReceiver> {
        Ok(UdpReceiver {
            socket: UdpSocket::bind(host)?,
        })
    }
    ///This function is used to start the UdpReceiver.
    ///It will create and start a InnerUdpReceiver struct.
    ///The joinhandle to this struct is returned by the run function.
    /// # Arguments
    /// * `bip_writer` - BipBufferWriter used by the InnerUdpReceiver.
    /// * `stats_data` - The struct used to store statistics data.
    /// # Returns
    /// `JoinHandle<()>` - The joinhandle to the InnerUdpReceiver.
    pub fn run(
        &self,
        bip_writer: BipBufferWriter,
        stats_data: Arc<StatsAllHandlers>,
    ) -> Result<()> {
        let socket = self.socket.try_clone()?;
        let inner_udp_receiver = InnerUdpReceiver::new(socket, bip_writer, stats_data);
        inner_udp_receiver.run();
        Ok(())
    }
}
