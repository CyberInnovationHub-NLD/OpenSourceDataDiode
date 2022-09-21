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
use log;
use std::net::SocketAddr;
use std::str::FromStr;
use syslog::Facility;

mod errors;

/// Setting syslog with a specific log level and processname.
/// The logging is sent with UDP
pub fn set_syslog(
    from_host_sys_log: &str,
    from_port_sys_log: &str,
    to_host_sys_log: &str,
    to_port_sys_log: &str,
    log_level_string: &str,
    process_name: &str,
) -> Result<()> {
    //Parse the socketsadresses
    let from_host: SocketAddr = format!("{}:{}", from_host_sys_log, from_port_sys_log)
        .parse()
        .chain_err(|| "Cant convert host and port to a socket addres")?;
    let to_host: SocketAddr = format!("{}:{}", to_host_sys_log, to_port_sys_log)
        .parse()
        .chain_err(|| "Cant convert host and port to a socket addres")?;

    //Create syslog with UDP
    syslog::init_udp(
        from_host,
        to_host,
        process_name.to_string(),
        //TODO: Is it the right facility?
        Facility::LOG_USER,
        log::Level::from_str(&log_level_string)
            .chain_err(|| "Cant convert log_level_string to loglevel")?
            .to_level_filter(),
    )?;
    Ok(())
}
