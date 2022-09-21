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
///Commandline arguments used to run the mock ingress.
#[derive(StructOpt)]
pub struct OptIngress {
    #[structopt(
        long = "socket_path",
        default_value = "/tmp/transport_to_handler",
        help = "Location of the socket"
    )]
    pub socket_path: String,

    #[structopt(long = "stats_server_address", default_value = "10.0.0.2")]
    pub host_stats_server: String,

    #[structopt(long = "stats_server_port", default_value = "8125")]
    pub port_stats_server: u16,
    ///From syslog server host
    #[structopt(long = "from_host_sys_log", default_value = "0.0.0.0")]
    pub from_host_sys_log: String,

    ///From syslog server port
    #[structopt(long = "from_port_sys_log", default_value = "8345")]
    pub from_port_sys_log: u16,

    ///To syslog udp host
    #[structopt(long = "to_host_sys_log", default_value = "127.0.0.1")]
    pub to_host_sys_log: String,

    ///To syslog udp port
    #[structopt(long = "to_port_sys_log", default_value = "8082")]
    pub to_port_sys_log: u16,

    ///Log level for logging
    #[structopt(long = "handler_name", default_value = "mock_handler_ingress")]
    pub handler_name: String,
}

///Commandline arguments used to run the mock egress.
#[derive(StructOpt)]
pub struct OptEgress {
    #[structopt(
        long = "socket_path",
        default_value = "/tmp/transport_to_handler",
        help = "Location of the socket"
    )]
    pub socket_path: String,

    #[structopt(long = "stats_server_address", default_value = "10.0.0.2")]
    pub host_stats_server: String,

    #[structopt(long = "stats_server_port", default_value = "8125")]
    pub port_stats_server: u16,
    ///From syslog server host
    #[structopt(long = "from_host_sys_log", default_value = "0.0.0.0")]
    pub from_host_sys_log: String,

    ///From syslog server port
    #[structopt(long = "from_port_sys_log", default_value = "8346")]
    pub from_port_sys_log: u16,

    ///To syslog udp host
    #[structopt(long = "to_host_sys_log", default_value = "127.0.0.1")]
    pub to_host_sys_log: String,

    ///To syslog udp port
    #[structopt(long = "to_port_sys_log", default_value = "8082")]
    pub to_port_sys_log: u16,

    ///Log level for logging
    #[structopt(long = "handler_name", default_value = "mock_handler_egress")]
    pub handler_name: String,
}
