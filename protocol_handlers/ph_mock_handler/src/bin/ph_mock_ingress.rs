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

use ph_mock_handler::*;
use socket_utils::socket_writer::SocketWriter;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread::JoinHandle;
use structopt::StructOpt;

pub struct MockHandlerIngress {
    path: String,
    should_stop: AtomicBool,
}

impl MockHandlerIngress {
    pub fn new(path: &str) -> MockHandlerIngress {
        MockHandlerIngress {
            path: path.to_string(),
            should_stop: AtomicBool::new(false),
        }
    }

    #[allow(clippy::while_immutable_condition)]
    pub fn run(&self) -> JoinHandle<()> {
        log::info!("Mock Handler Ingress started");
        let mut writer =
            SocketWriter::start_listening(&self.path).expect("cant create socket writer");
        let should_stop = self.should_stop.load(Ordering::SeqCst);
        std::thread::spawn(move || {
            let mut print_counter = 0;
            while !should_stop {
                writer
                    .send_data(&mut [0; 65500])
                    .expect("Error while sending data"); //write data with max payload size
                if print_counter > 10_000 {
                    log::info!("Data sent by Mock Handler Ingress");
                    print_counter = 0;
                }
                print_counter += 1;
            }
            writer.stop();
        })
    }

    pub fn stop(&self) {
        self.should_stop.store(true, Ordering::SeqCst);
        log::info!("Mock Handler Ingress is stopping");
    }
}

///Creates an IngressMockHandler struct.
///This struct starts sending UDP packets to the socket path supplied in the arguments.
fn main() {
    let opt = arguments::OptIngress::from_args();
    set_syslog(
        opt.from_host_sys_log,
        opt.from_port_sys_log.to_string(),
        opt.to_host_sys_log,
        opt.to_port_sys_log.to_string(),
    );
    let ingress = MockHandlerIngress::new(&opt.socket_path);
    let ingress_handle = ingress.run();
    ingress_handle.join().expect("Error joining thread!");
}
