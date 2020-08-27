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

use crate::errors::ErrorKind::UdpSocketError;
use crate::errors::*;
use crate::tx::message_split::split_and_send_data;
use crate::tx::special_message::*;
use spsc_bip_buffer::BipBufferReader;
use statistics_handler::*;
use std::net::UdpSocket;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::JoinHandle;

pub struct UdpSender {
    socket: UdpSocket,
    reader: Arc<Mutex<BipBufferReader>>,
    should_stop: Arc<AtomicBool>,
    send_delay_ms: u64,
    stats_data: Arc<StatsAllHandlers>,
}

impl UdpSender {
    pub fn new(
        host: &str,
        reader: BipBufferReader,
        send_delay_ms: u64,
        stats_data: Arc<StatsAllHandlers>,
    ) -> Result<UdpSender> {
        Ok(UdpSender {
            socket: { init_socket(host)? },
            reader: Arc::new(Mutex::new(reader)),
            should_stop: Arc::new(AtomicBool::new(false)),
            send_delay_ms,
            stats_data,
        })
    }
    ///This function is used to start the UdpSender on a seperate thread.
    ///The joinhandle to this thread is returned.
    /// # Arguments
    /// * `rec_addr` - The address of the UdpReceiver.
    /// # Returns
    /// `JoinHandle<()>` - The JoinHandle of the thread that is started.
    pub fn run(&self, rec_addr: &str) -> std::io::Result<JoinHandle<()>> {
        let socket = self.socket.try_clone()?;
        let reader_mutex = Arc::clone(&self.reader);
        let should_stop = Arc::clone(&self.should_stop);
        let receiver_addr = String::from(rec_addr);
        let send_delay_ms = self.send_delay_ms;
        let stats_data = self.stats_data.clone();
        std::thread::Builder::new()
            .name("udp_sender_thread".into())
            .spawn(move || {
                clean_unwrap(
                    udp_sender_thread(
                        socket,
                        receiver_addr,
                        should_stop,
                        reader_mutex,
                        send_delay_ms,
                        stats_data,
                    )
                    .chain_err(|| "Error in udp_sender thread"),
                )
            })
    }

    ///This function is used to stop the UdpSender thread.
    ///It will also send shutdown messages to the UdpReceiver.
    pub fn stop(&self) {
        send_shutdown_messages(&self.socket);
        self.should_stop.store(true, Ordering::SeqCst);
        log::info!("sender is stopping.");
    }
}

///This function is used to initialize the socket.
///It sets the BroadCast flag.
fn init_socket(host: &str) -> Result<UdpSocket> {
    let socket = match UdpSocket::bind(host) {
        Ok(socket) => socket,
        Err(e) => return Err(UdpSocketError(e.to_string()).into()),
    };
    socket
        .set_broadcast(true)
        .chain_err(|| UdpSocketError("Error whil setting broadcast flag for socket".to_string()))?;
    Ok(socket)
}

pub fn udp_sender_thread(
    socket: UdpSocket,
    receiver_addr: String,
    should_stop: Arc<AtomicBool>,
    reader_mutex: Arc<Mutex<BipBufferReader>>,
    send_delay_ms: u64,
    stats_data: Arc<StatsAllHandlers>,
) -> Result<()> {
    socket
        .connect(receiver_addr.to_string())
        .chain_err(|| format!("Failed connect to socket address: {}", receiver_addr))?;
    let mut sequence_number: u32 = 0;
    send_startup_messages(&socket, &mut sequence_number);
    while !(should_stop.load(Ordering::SeqCst)) {
        let mut reader = reader_mutex.lock().expect("Error locking mutex");
        split_and_send_data(
            &socket,
            &mut reader,
            &mut sequence_number,
            send_delay_ms,
            stats_data.clone(),
        );
    }
    Ok(())
}
