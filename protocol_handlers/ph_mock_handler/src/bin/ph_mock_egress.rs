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

use ph_mock_handler::set_syslog;
use ph_mock_handler::*;
use socket_utils::socket_reader::SocketReader;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread::JoinHandle;
use structopt::StructOpt;

pub struct MockHandlerEgress {
    path: String,
    should_stop: AtomicBool,
}

impl MockHandlerEgress {
    pub fn new(path: &str) -> MockHandlerEgress {
        MockHandlerEgress {
            path: path.to_string(),
            should_stop: AtomicBool::new(false),
        }
    }
    #[allow(clippy::while_immutable_condition)]
    pub fn run(&self) -> JoinHandle<()> {
        log::info!("Mock Handler Egress started");
        let mut reader = SocketReader::new(&self.path).expect("Can't create socket reader");
        let should_stop = self.should_stop.load(Ordering::SeqCst);
        std::thread::spawn(move || {
            let mut buffer: [u8; 65508] = [0; 65508]; //max payload size + 8 length bytes
            let mut print_counter = 0;
            while !should_stop {
                reader
                    .receive_data(&mut buffer)
                    .expect("Error while receiving data");
                if print_counter > 10_000 {
                    log::info!("Data received by Mock Handler Egress");
                    print_counter = 0;
                }
                print_counter += 1;
            }
            reader.stop().expect("Cant stop reader");
        })
    }
    pub fn stop(&self) {
        self.should_stop.store(true, Ordering::SeqCst);
        log::info!("Mock Handler Egress is stopping");
    }
}
///Creates an EgressMockHandler struct.
///This struct starts reading data from the socket path supplied in the arguments.
fn main() {
    let opt = arguments::OptEgress::from_args();
    set_syslog(
        opt.from_host_sys_log,
        opt.from_port_sys_log.to_string(),
        opt.to_host_sys_log,
        opt.to_port_sys_log.to_string(),
    );
    let egress = MockHandlerEgress::new(&opt.socket_path);
    let egress_handle = egress.run();
    egress_handle.join().expect("Error joining thread!");
}
