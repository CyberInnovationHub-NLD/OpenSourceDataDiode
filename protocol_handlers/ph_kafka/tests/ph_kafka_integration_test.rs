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

use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
use kafka::producer::{Producer, Record};
use log::{error, trace};
use rand::Rng;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::result::Result;
use std::str;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::{thread, time};

const TOPICNAME: &str = "TestTopic";
const TOPICNAME2: &str = "TestTopic";
const PORT_KAFKA_SERVER: u16 = 9092;
const MILISECONDS_TO_WAIT: u64 = 200;
const PORT_STATSD_SERVER: u16 = 8125;

#[test]
#[ignore]
fn kafka_integration_test() {
    env_logger::init();
    let mut host_kafka_server_ingress: String = "192.168.1.2".to_string();
    let mut host_kafka_server_egress: String = "192.168.1.5".to_string();

    let vars = std::env::vars();

    for (key, value) in vars {
        match key.as_ref() {
            "HOSTKAFKAINGRESS" => host_kafka_server_ingress = value,
            "HOSTKAFKAEGRESS" => host_kafka_server_egress = value,
            _ => (),
        };
    }

    log::info!("start test");

    //init statsd_handler and run in a different thread to collect de statsddata.
    let statsd_handler = StadsDHandler::new(PORT_STATSD_SERVER);
    statsd_handler.run();

    //create groupless consumer. All data send to kafka after init this consumer this will be consumed by this consumer
    let mut consumer = setup_consumer(TOPICNAME2, &host_kafka_server_egress, PORT_KAFKA_SERVER)
        .expect("error while setting consumer");

    //produce data
    let host_port = format!("{}:{}", &host_kafka_server_ingress, PORT_KAFKA_SERVER);
    let mut producer = Producer::from_hosts(vec![host_port])
        .create()
        .expect("Can't create producer");
    //let test_data = TestData::new_size(TOPICNAME, MILISECONDS_TO_WAIT, 3_221);
    let test_data = TestData::new_size(TOPICNAME, MILISECONDS_TO_WAIT, 3_221_225_472);
    //let test_data = TestData::new_size(TOPICNAME, MILISECONDS_TO_WAIT, 200_000_000_000);
    log::info!("sending testdata to kafka...");
    let test_data_clone1 = test_data.clone();
    let test_data_clone2 = test_data.clone();
    let _producer_thread = thread::Builder::new()
        .name("producer_thread".into())
        .spawn(move || {
            test_data.send(&mut producer);
            log::info!("testdata is sent to kafka");
        });

    //Consume data
    //let test_data = TestData::new(TOPICNAME2, MILISECONDS_TO_WAIT);
    log::info!("Start receiving data..");
    test_data_clone1.receive(&mut consumer, TOPICNAME2.to_string());
    log::info!("All data received...");

    //a sleep to make sure all statsd data is collected
    thread::sleep(time::Duration::from_millis(5000));

    //stop statds and check all the data
    statsd_handler.stop_and_check(test_data_clone2.data_times_size);
}

#[derive(Clone)]
pub struct TestData {
    pub data_times_size: Vec<(usize, usize)>,
    topic: String,
    miliseconds_to_wait: u64,
}
impl TestData {
    pub fn new(topic: &str, miliseconds_to_wait: u64) -> TestData {
        //init test data
        let data_times_size = vec![
            (10, 2),
            (10, 10),
            (10, 100),
            (40, 1_000),
            (10, 10_000),
            (10, 100_000),
            //(10, 900_000),
        ];
        TestData {
            data_times_size,
            topic: topic.to_string(),
            miliseconds_to_wait,
        }
    }
    pub fn new_size(topic: &str, miliseconds_to_wait: u64, size: usize) -> TestData {
        let mut totalsize = 0;
        let mut data_times_size: Vec<(usize, usize)> = Vec::new();
        let mut rng = rand::thread_rng();
        while totalsize < size {
            let random_size = rng.gen_range(0, 100_000);
            let random_times = rng.gen_range(0, 1000);
            data_times_size.push((random_times, random_size));
            totalsize += random_size * random_times;
        }
        log::info!("generated testdata(times, size): {:?}", data_times_size);
        TestData {
            data_times_size,
            topic: topic.to_string(),
            miliseconds_to_wait,
        }
    }
    pub fn send(&self, producer: &mut Producer) {
        //send testdata to kafka
        let mut i = 0;
        for (times, size) in &self.data_times_size {
            //sleep to do simulate burst
            thread::sleep(time::Duration::from_millis(self.miliseconds_to_wait));
            let mut data: Vec<u8> = vec![0; *size];
            for _ in 0..*times {
                //set first byte to a counter. If i = 255 it reset to 0
                data[0] = i;
                i = i.wrapping_add(1);
                producer
                    .send(&Record::from_value(&self.topic, data.clone()))
                    .expect("Can't send kafka message");
            }
        }
    }
    pub fn receive(&self, consumer: &mut Consumer, topic: String) {
        let mut test_passed = true;
        let mut total_out_of_order_messages = 0;
        let mut input_data_iter = self.data_times_size.iter();
        //counter to check messages are the same order
        let mut i: u8 = 0;
        let mut kafka_output_size = Vec::new();
        let mut input_data = match input_data_iter.next() {
            Some(v) => v,
            None => panic!("Error while loading test data"),
        };

        //keep looping till all testdata is checked
        let mut next_testdata_available = true;
        while next_testdata_available {
            //loop till minimum of one testdata sequence
            while kafka_output_size.len() < input_data.0 {
                match consumer.poll() {
                    Ok(v) => {
                        for ms in v.iter() {
                            for m in ms.messages() {
                                trace!("Offset: {}, Message: {:?} I: {}", m.offset, m.value[0], i,);

                                match consumer.consume_message(&topic, 0, m.offset) {
                                    Ok(_) => (),
                                    Err(e) => {
                                        error!("Cannot consume kafka message: {}", e);
                                        test_passed = false;
                                    }
                                }
                                match consumer.commit_consumed() {
                                    Ok(_) => trace!("{} is consumed to kafka", m.offset),
                                    Err(e) => {
                                        error!("Cannot commit consumed kafka message: {}", e);
                                        test_passed = false;
                                    }
                                }
                                //push the lengt of the message to a vec
                                kafka_output_size.push(m.value.len());

                                //check order of messages
                                if m.value[0] != i {
                                    log::warn!("Message out of order!");
                                    total_out_of_order_messages += 1;
                                    i = m.value[0]
                                }
                                assert_eq!(m.value[0], i, "message out of order");
                                i = i.wrapping_add(1);
                            }
                        }
                    }
                    Err(e) => {
                        error!("error while polling kafka: {}", e);
                        test_passed = false;
                    }
                }
            }
            //checking buffered data. If buffer is not big enough it stops.
            while kafka_output_size.len() >= input_data.0 {
                //get one sequence out of the buffer
                let tempvec = kafka_output_size[0..input_data.0].to_vec();
                kafka_output_size = kafka_output_size[input_data.0..].to_vec();

                //check if sequence is correct length
                for output_size in tempvec {
                    assert_eq!(output_size, input_data.1 as usize);
                }
                //if data available, reset new message set. Else stop testing
                match input_data_iter.next() {
                    Some(v) => input_data = v,
                    None => next_testdata_available = false,
                };
            }
        }

        if total_out_of_order_messages > 0 {
            log::warn!("Message out of order: {}", total_out_of_order_messages);
            log::warn!("Lost a least {} packets", total_out_of_order_messages);
            assert!(false);
        }

        assert!(test_passed);
        log::info!("Data received succesfull!");
    }
}

fn setup_consumer(topic: &str, host: &String, port: u16) -> Result<Consumer, kafka::error::Error> {
    let host_port = format!("{}:{}", host, port);
    Consumer::from_hosts(vec![host_port])
        .with_topic(topic.to_owned())
        //for testing only the latests message needed
        .with_fallback_offset(FetchOffset::Latest)
        //testing with testgroup. maybe in the future test with other testgroups
        // .with_group("".to_owned())
        .with_offset_storage(GroupOffsetStorage::Kafka)
        .with_fetch_max_bytes_per_partition(1_000_000)
        .create()
}

pub struct StadsDHandler {
    stats_data: Arc<StadsData>,
    should_stop: Arc<AtomicBool>,
    port: u16,
}

impl StadsDHandler {
    pub fn new(port: u16) -> StadsDHandler {
        StadsDHandler {
            stats_data: Arc::new(StadsData::default()),
            should_stop: Arc::new(AtomicBool::new(false)),
            port,
        }
    }

    pub fn run(&self) {
        let should_stop = Arc::clone(&self.should_stop);
        let stats_data = Arc::clone(&self.stats_data);
        let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), self.port);
        let socket = UdpSocket::bind(addr)
            .ok()
            .expect("Can't parse udp socket in stadsD Handler");
        let mut buf = [0; 65507];

        let _kafka_poll_bipwriter = thread::Builder::new().name("stats_tread".into()).spawn(
            move || {
            while !(should_stop.load(Ordering::SeqCst)) {
                let (len, _) = match socket.recv_from(&mut buf) {
                    Ok(r) => r,
                    Err(_) => panic!("Could not read UDP socket."),
                };
                let bytes = Vec::from(&buf[..len]);
                let text = str::from_utf8(&bytes);
                let objects = text.expect("Can't convert stads data to utf8").split("\n");
                for object in objects {
                      //  println!("{}", object);
                    let object_vec: Vec<&str> = object.split(":").collect();
                        if object_vec.len() < 2 {
                            log::trace!(
                                "error while reading stats udp packet. First part of message: {}",
                                object_vec[0]
                            )
                        } else {
                            match object_vec[1].split("|").collect::<Vec<&str>>()[0].parse() {
                                Ok(number) => {
                    match object_vec[0] {
                        //e => log:info!("{} : {}", e, number),
                        //kafka ingress
                                        "osdd.1.ingress.TestTopic.ph.kafka.in.packets" => {
                            stats_data
                                .ph_kafka_ingress_messages_counter
                                .fetch_add(number, Ordering::Relaxed);
                        }
                                        "osdd.1.ingress.TestTopic.ph.kafka.in.bytes" => {
                            stats_data
                                .ph_kafka_ingress_message_bytes_gauge
                                .fetch_add(number, Ordering::SeqCst);
                        }
                        //kafka egress
                                        "osdd.1.egress.TestTopic.ph.kafka.out.bytes" => {
                            stats_data
                                .ph_kafka_egress_message_bytes_gauge
                                .fetch_add(number, Ordering::Relaxed);
                        }
                                        // "ph_kafka.egress.packed_message_bytes_gauge" => {
                                        //     stats_data
                                        //         .ph_kafka_egress_packed_message_bytes_gauge
                                        //         .fetch_add(number, Ordering::Relaxed);
                                        // }
                                        "osdd.1.egress.TestTopic.ph.kafka.out.packets" => {
                            stats_data
                                .ph_kafka_egress_messages_counter
                                .fetch_add(number, Ordering::Relaxed);
                        }
                                        "osdd.1.egress.TestTopic.ph.kafka.dropped.packets" => {
                            stats_data
                                .ph_kafka_egress_sending_to_kafka_error_counter
                                .fetch_add(number, Ordering::Relaxed);
                        }
                                        "osdd.1.egress.TestTopic.ph.kafka.dropped.packets.bytes" => {
                            stats_data
                                .ph_kafka_egress_sending_on_a_full_channel_counter
                                .fetch_add(number, Ordering::Relaxed);
                        }
                        //transport send
                                        "ingress_TestTopic.transport1.bytes_sent" => {
                            stats_data
                                .transport_udp_sender_bytes_sent
                                .fetch_add(number, Ordering::Relaxed);
                        }
                                        "egress_TestTopic.transport1.bytes_received" => {
                            stats_data
                                .transport_udp_receiver_bytes_received
                                .fetch_add(number, Ordering::Relaxed);
                        }
                                        "egress_TestTopic.transport1.packetloss" => {
                            stats_data
                                .transport_udp_receiver_packetloss
                                .store(number, Ordering::Relaxed);
                        }
                                        "ingress_TestTopic.transport1.sequence_number" => {}
                        _ => (),
                    };
                }
                                Err(e) => log::trace!("error while converting to u32: {}", e),
                            }
                        }
                    }
            }
            },
        );
    }

    pub fn stop_and_check(&self, test_data: Vec<(usize, usize)>) {
        log::info!("Checking for stats");
        self.should_stop.store(true, Ordering::Relaxed);
        let mut total_messages: u32 = 0;
        let mut total_size: u32 = 0;
        for (times, size) in test_data {
            total_messages += times as u32;
            total_size += (size * times) as u32;
        }
        assert_eq!(
            total_messages,
            self.stats_data
                .ph_kafka_ingress_messages_counter
                .load(Ordering::Relaxed),
            "StadsD ph_kafka_ingress_messages_counter wrong"
        );
        assert_eq!(
            total_size,
            self.stats_data
                .ph_kafka_ingress_message_bytes_gauge
                .load(Ordering::Relaxed),
            "stadsD ph_kafka_ingress_bytes_gauge wrong"
        );
        assert_eq!(
            total_size,
            self.stats_data
                .ph_kafka_egress_message_bytes_gauge
                .load(Ordering::Relaxed),
            "stadsD ph_kafka_egress_bytes_gauge wrong"
        );
        // assert!(
        //     total_size
        //         < self
        //             .stats_data
        //             .ph_kafka_egress_packed_message_bytes_gauge
        //             .load(Ordering::Relaxed),
        //     "Total size: {}, packed size: {}",
        //     total_size,
        //     self.stats_data
        //         .ph_kafka_egress_packed_message_bytes_gauge
        //         .load(Ordering::Relaxed)
        // );
        assert_eq!(
            total_messages,
            self.stats_data
                .ph_kafka_egress_messages_counter
                .load(Ordering::Relaxed),
            "statsD ph_kafka_egress_messages_counter wrong"
        );
        assert_eq!(
            0,
            self.stats_data
                .ph_kafka_egress_sending_to_kafka_error_counter
                .load(Ordering::Relaxed),
            "Sending to kafka error"
        );
        assert_eq!(
            0,
            self.stats_data
                .ph_kafka_egress_sending_on_a_full_channel_counter
                .load(Ordering::Relaxed),
            "Sending to full channel"
        );
        assert_eq!(
            0,
            self.stats_data
                .transport_udp_receiver_packetloss
                .load(Ordering::Relaxed),
            "{} packet lost!",
            self.stats_data
                .transport_udp_receiver_packetloss
                .load(Ordering::Relaxed)
        );
    }
}

#[derive(Default)]
pub struct StadsData {
    ph_kafka_ingress_messages_counter: AtomicU32,
    ph_kafka_ingress_message_bytes_gauge: AtomicU32,
    //kafka egress
    ph_kafka_egress_message_bytes_gauge: AtomicU32,
    // ph_kafka_egress_packed_message_bytes_gauge: AtomicU32,
    ph_kafka_egress_messages_counter: AtomicU32,
    ph_kafka_egress_sending_to_kafka_error_counter: AtomicU32,
    ph_kafka_egress_sending_on_a_full_channel_counter: AtomicU32,
    //transport
    transport_udp_sender_bytes_sent: AtomicU32,
    transport_udp_receiver_bytes_received: AtomicU32,
    transport_udp_receiver_packetloss: AtomicU32,
}
