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

use kafka::producer::{Producer, Record, RequiredAcks};

use std::io;
use std::time::Duration;

#[derive(StructOpt)]
struct Opt {
    ///send x amount of message
    #[structopt(short, required = false, default_value = "0")]
    auto: usize,

    ///set size of multiple message that will be sent
    #[structopt(short, required = false, default_value = "100")]
    size: usize,

    #[structopt(short)]
    topic_name: String,

    #[structopt(short)]
    host_kafka_server: String,

    //maybe check for number
    #[structopt(short)]
    port_kafka_server: String,
}

fn main() {
    let opt = Opt::from_args();

    let host_port = format!("{}:{}", &opt.host_kafka_server, &opt.port_kafka_server);

    let mut producer = Producer::from_hosts(vec![host_port])
        .with_ack_timeout(Duration::from_secs(1))
        .with_required_acks(RequiredAcks::One)
        .create()
        .expect("Can't create producer");

    if opt.auto != (0 as usize) {
        let mut record_array: Vec<Record<(), Vec<u8>>> = Vec::with_capacity(opt.auto);
        let mut bytes_counter = 0;
        for _ in 0..opt.auto {
            let vector: Vec<u8> = vec![0; opt.size];
            record_array.push(Record::from_value(&opt.topic_name, vector));
            bytes_counter += opt.size;

            if bytes_counter > 500_000 {
                producer
                    .send_all(&record_array)
                    .expect("Can't send kafka messages");
                record_array.clear();
            }
        }
        producer
            .send_all(&record_array)
            .expect("Can't send kafka messages");
    } else {
        loop {
            let mut input = String::new();

            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            //NOTE: Use of trim. For the enter on the end. But removes also whitespace at beginning
            producer
                .send(&Record::from_value(
                    &opt.topic_name,
                    input.trim().as_bytes(),
                ))
                .expect("Can't send kafka message");
        }
    }
}
