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

use log::trace;
use std::result::Result;
//use std::time::Instant;

use structopt::StructOpt;

use std::str;

use kafka::consumer::{Consumer, FetchOffset};
use syslog::Facility;

#[derive(StructOpt)]
struct Opt {
    #[structopt(short)]
    topic_name: String,

    #[structopt(short)]
    host_kafka_server: String,

    //maybe check for number
    #[structopt(short)]
    port_kafka_server: String,

    ///Display data per message
    #[structopt(short)]
    read_per_message: bool,
}

fn main() {
    syslog::init(Facility::LOG_USER, log::LevelFilter::Warn, None).expect("Can't init syslog");
    let opt = Opt::from_args();

    let consumer_error = setup_consumer(
        &opt.topic_name,
        &opt.host_kafka_server,
        &opt.port_kafka_server,
    );

    let mut consumer = match consumer_error {
        Ok(v) => v,
        Err(e) => panic!("The program paniced with error: {}", e),
    };

    let mut bytes_count = 0;

    //let start_time = Instant::now();
    if opt.read_per_message {
        loop {
            match consumer.poll() {
                Ok(v) => {
                    for ms in v.iter() {
                        for m in ms.messages() {
                            match str::from_utf8(m.value) {
                                Ok(v) => println!(
                                    "message received: {} | Length is: {}",
                                    v,
                                    m.value.len()
                                ),
                                Err(_) => println!("message received, but can't convert to utf8"),
                            }
                            // println!(
                            //     "Message: {:?}, Offset: {}, Message in bytes: {:?}",
                            //     m.value,
                            //     m.offset,
                            //     m.value.len()
                            // );
                            match consumer.consume_message(&opt.topic_name, 0, m.offset) {
                                Ok(_) => (),
                                Err(e) => trace!("error: {}", e),
                            }
                        }
                    }
                }
                Err(e) => log::error!("error while polling kafka: {}", e),
            }
            match consumer.commit_consumed() {
                Ok(_) => (),
                Err(e) => trace!("error: {}", e),
            }
        }
    }
    loop {
        let mut new_data = false;
        match consumer.poll() {
            Ok(v) => {
                for ms in v.iter() {
                    for m in ms.messages() {
                        bytes_count += m.value.len();
                        new_data = true;
                        trace!(
                            "Offset: {}, Message in bytes: {:?}",
                            m.offset,
                            m.value.len()
                        );

                        match consumer.consume_message(&opt.topic_name, 0, m.offset) {
                            Ok(_) => (),
                            Err(e) => trace!("error: {}", e),
                        }
                    }
                }
            }
            Err(e) => log::error!("error while polling kafka: {}", e),
        }
        if new_data {
            println!("total bytes received: {}", bytes_count,);
        }
        match consumer.commit_consumed() {
            Ok(_) => (),
            Err(e) => trace!("error: {}", e),
        }
    }
}

/// Setup a kafka consumer
/// # Arguments
/// * `topic` - Topic name
/// * `host` - Host of the kafka server
/// * `port` - Server port of the kafka server
/// # Returns
/// * `Consumer` - Kafka Consumer
pub fn setup_consumer(
    topic: &str,
    host: &str,
    port: &str,
) -> Result<Consumer, kafka::error::Error> {
    let host_port = format!("{}:{}", host, port);
    Consumer::from_hosts(vec![host_port])
        .with_topic(topic.to_owned())
        //[OSSD-17]: Alle vs laatste berichten
        .with_fallback_offset(FetchOffset::Latest)
        .with_fetch_max_bytes_per_partition(1_000_000)
        //[OSDD-22]: Kafka consumer group configureerbaar
        .with_group("TestGroup2".to_owned())
        //.with_offset_storage(GroupOffsetStorage::Kafka)
        .create()
}
