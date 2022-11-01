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

use structopt::StructOpt;

///This struct contains all structopt definitions used by the UdpReceiver.
#[derive(StructOpt)]
pub struct OptReceiver {
    #[structopt(
        long = "socket_path",
        default_value = "/tmp/transport_to_handler",
        help = "Location of the socket"
    )]
    ///The path used for the unix domain socket.
    pub socket_path: String,

    #[structopt(
        long = "receiver_address",
        default_value = "192.168.0.1",
        help = "Address the receiver is hosted on."
    )]
    ///The address of the UdpReceiver.
    pub receiver_addr: String,

    #[structopt(
        long = "receiver_port",
        default_value = "1234",
        help = "Port the receiver should be listening on."
    )]
    ///The port of the UdpReceiver.
    pub receiver_port: u16,

    #[structopt(long = "stats_server_address", default_value = "10.0.0.2")]
    ///The address of the stats server.
    pub host_stats_server: String,

    #[structopt(long = "stats_server_port", default_value = "8125")]
    ///The port of the stats server.
    pub port_stats_server: u16,

    ///The maximum amount of elements the bip buffer can store.
    ///The size of a single element is 1Mb.
    #[structopt(long = "bip_buffer_element_count", default_value = "10")]
    pub bip_buffer_element_count: usize,

    ///From syslog server host
    #[structopt(long = "from_host_sys_log", default_value = "0.0.0.0")]
    pub from_host_sys_log: String,

    ///From syslog server port
    #[structopt(long = "from_port_sys_log", default_value = "8342")]
    pub from_port_sys_log: u16,

    ///To syslog udp host
    #[structopt(long = "to_host_sys_log", default_value = "127.0.0.1")]
    pub to_host_sys_log: String,

    ///To syslog udp port
    #[structopt(long = "to_port_sys_log", default_value = "8082")]
    pub to_port_sys_log: u16,

    ///Log level for logging
    #[structopt(long = "log_level", default_value = "Warn")]
    pub log_level: String,

    ///Name of the handler
    #[structopt(long = "handler_name", default_value = "transport_udp_receive")]
    pub handler_name: String,
}

impl OptReceiver {
    ///This function is used to log the complete configuration of the UdpReceiver.
    pub fn log_config_info(&self) {
        log::info!("---------------------------------------\r\n");
        log::info!(
            "Starting Receiver at {}\r\n",
            format!("{}:{}", &self.receiver_addr, &self.receiver_port)
        );
        log::info!(
            "Sending statistic data to {}\r\n",
            format!("{}:{}", &self.host_stats_server, &self.port_stats_server)
        );
        log::info!("Sharing data over socket at {}\r\n", &self.socket_path);
        log::info!(
            "Using syslog for logging at {}\r\n",
            format!("{}:{}", &self.from_host_sys_log, &self.from_port_sys_log)
        );
        log::info!("Log level is {}", &self.log_level);
        log::info!("---------------------------------------\r\n\r\n");
    }
}
