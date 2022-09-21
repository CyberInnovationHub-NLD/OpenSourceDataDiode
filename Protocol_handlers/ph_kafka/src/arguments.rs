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
///Commandline arguments used to run ph_kafka_ingress.
#[derive(StructOpt)]
pub struct OptIngress {
    #[structopt(
        long = "socket_path",
        default_value = "/tmp/handler_to_transport",
        help = "Location of the socket"
    )]
    pub socket_path: String,

    ///Max bytes per message settings for consumer
    #[structopt(long = "max_bytes_per_partition", default_value = "1000000")]
    pub max_bytes_per_partition: usize,

    ///StatsD server host.
    #[structopt(long = "stats_server_address", default_value = "127.0.0.1")]
    pub host_stats_server: String,

    ///StatsD server port.
    #[structopt(long = "stats_server_port", default_value = "8125")]
    pub port_stats_server: u16,

    ///Topic to read from the kafka server
    #[structopt(short, long = "topic_name", default_value = "TestTopic")]
    pub topic_name: String,

    ///kafka server host
    #[structopt(short, long = "host_kafka_server", default_value = "10.0.0.1")]
    pub host_kafka_server: String,

    ///Kafka server port
    #[structopt(short, long = "port_kafka_server", default_value = "9092")]
    pub port_kafka_server: u16,

    ///From syslog server host
    #[structopt(long = "from_host_sys_log", default_value = "0.0.0.0")]
    pub from_host_sys_log: String,

    ///From syslog server port
    #[structopt(long = "from_port_sys_log", default_value = "8127")]
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

    ///Log level for logging
    #[structopt(long = "handler_name", default_value = "ph_kafka_ingress")]
    pub handler_name: String,

    ///The maximum amount of elements the bip buffer can store.
    ///The size of a single element is 1Mb.
    #[structopt(long = "bip_buffer_element_count", default_value = "2")]
    pub bip_buffer_element_count: usize,
}

///Commandline arguments used to run ph_kafka_egress.
#[derive(StructOpt)]
pub struct OptEgress {
    #[structopt(
        long = "socket_path",
        default_value = "/tmp/transport_to_handler",
        help = "Location of the socket"
    )]
    pub socket_path: String,

    ///StatsD server host.
    #[structopt(long = "stats_server_address", default_value = "localhost")]
    pub host_stats_server: String,

    ///StatsD server port.
    #[structopt(long = "stats_server_port", default_value = "8125")]
    pub port_stats_server: u16,

    ///kafka server host
    #[structopt(short, long = "host_kafka_server", default_value = "10.0.0.2")]
    pub host_kafka_server: String,

    ///Kafka server port
    #[structopt(short, long = "port_kafka_server", default_value = "9092")]
    pub port_kafka_server: u16,

    ///The maximum amount of elements the bip buffer can store.
    ///The size of a single element is 1Mb.
    #[structopt(long = "bip_buffer_element_count", default_value = "10")]
    pub bip_buffer_element_count: usize,

    ///Topic to replace
    #[structopt(short, long = "in_replacement", default_value = "TestTopic")]
    //Use this command to replace a specific topic name. This is the inputlist
    pub in_replacement: String,

    ///replace topic
    #[structopt(short, long = "out_replacement", default_value = "TestTopic2")]
    //Use this command to replace a specific topic name. This is the outputlist
    pub out_replacement: String,

    ///From syslog server host
    #[structopt(long = "from_host_sys_log", default_value = "0.0.0.0")]
    pub from_host_sys_log: String,

    ///From syslog server port
    #[structopt(long = "from_port_sys_log", default_value = "8129")]
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

    ///Log level for logging
    #[structopt(long = "handler_name", default_value = "ph_kafka_egress")]
    pub handler_name: String,
}
