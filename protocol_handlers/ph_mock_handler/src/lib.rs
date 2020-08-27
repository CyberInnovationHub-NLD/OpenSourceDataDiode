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

use std::net::SocketAddr;
use syslog::Facility;

pub fn set_syslog(
    from_host_sys_log: String,
    from_port_sys_log: String,
    to_host_sys_log: String,
    to_port_sys_log: String,
) {
    //init sys log
    let from_host: SocketAddr = format!("{}:{}", from_host_sys_log, from_port_sys_log)
        .parse()
        .expect("Can't convert stats server from config file to a socketadres");
    let to_host: SocketAddr = format!("{}:{}", to_host_sys_log, to_port_sys_log)
        .parse()
        .expect("Can't convert stats server from config file to a socketadres");
    match syslog::init_udp(
        from_host,
        to_host,
        "".to_string(),
        Facility::LOG_USER,
        log::LevelFilter::Info,
    ) {
        Ok(_) => log::info!("send syslog to from {} to {}", from_host, to_host),
        Err(e) => panic!("cant init syslog, error: {}", e),
    }
}
pub mod arguments;
