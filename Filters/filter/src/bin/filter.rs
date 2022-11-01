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

use bip_utils::read_from_bip_buffer;
use filter::errors::*;
use filter::*;
use logging::*;
use socket_utils::buffered_socket_reader::BufferedSocketReader;
use socket_utils::buffered_socket_writer::BufferedSocketWriter;
use spsc_bip_buffer::bip_buffer_with_len;
use statistics_handler::*;
use std::thread;
use structopt::StructOpt;

fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            log::error!("{}", s);
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            log::error!("{}", s);
        } else {
            log::error!(
                "No payload available in panic info! printing panic info: {}",
                panic_info
            );
        }
        std::process::exit(1);
    }));

    filter().chain_unwrap();
}

/// This filter checks for the first bytes of the incoming data. If it matches the configured "word_to_filter" then it drops the data.
/// If the data is corrupted and cannot be read as KafkaMessage then the data will always drop.
fn filter() -> Result<()> {
    let opt = arguments::OptIngress::from_args();
    set_syslog(
        &opt.from_host_sys_log,
        &opt.from_port_sys_log.to_string(),
        &opt.to_host_sys_log,
        &opt.to_port_sys_log.to_string(),
        &opt.log_level,
        &opt.handler_name,
    )
    .chain_err(|| "Error initializing syslog")?;
    log::info!("start {}", &opt.handler_name);

    //Start stats thread
    let stats_addr = format!("{}:{}", opt.host_stats_server, opt.port_stats_server);
    let stats: StatsdClient<StatsAllHandlers> =
        StatsdClient::<StatsAllHandlers>::new_with_custom_fields(Some("filtered"), None);
    stats
        .run(stats_addr, opt.handler_name)
        .chain_err(|| "Error while running statitics")?;

    //Create multiple clones to the stats_data for the seperate threads.
    let stats_data = stats.get_data_clone();
    let stats_data2 = stats.get_data_clone();
    let stats_data3 = stats.get_data_clone();

    //Create bipbuffers with the a size of 1Mb times the incoming bip_bupffer_element_count in argument
    let (bip_writer_first, mut bip_reader_first) =
        bip_buffer_with_len(opt.bip_buffer_element_count * BUFFER_SIZE_BYTES);
    let (mut bip_writer_second, mut bip_reader_second) =
        bip_buffer_with_len(opt.bip_buffer_element_count * BUFFER_SIZE_BYTES);

    let mut socket_reader = BufferedSocketReader::new(&opt.socket_path_in, bip_writer_first)
        .chain_err(|| "Error while creating socket reader")?;
    let mut socket_writer = BufferedSocketWriter::start_listening(&opt.socket_path_out)
        .chain_err(|| "Error creating socket writer")?;

    //3 threads:
    //- get_data_from_socket_send_to_bip_buffer
    //- filtering
    //- bipreader_socketwriter

    // {UNIX_DOMAIN_SOCKET} <-- get_data_from_socket_send_to_bip_buffer --> {BIPBUFFER_FIRST} --> filtering --> {BIPBUFFER_SECOND} <-- bipreader_socketwriter --> {UNIX_DOMAIN_SOCKET}

    //Clone word_to_filter for the filtering thread
    let word_to_filter = opt.word_to_filter;

    let filtering = thread::Builder::new()
        .name("filtering".into())
        .spawn(move || {
            let mut buffer = [0; BUFFER_SIZE_BYTES];
            loop {
                let element_length = read_from_bip_buffer(&mut bip_reader_first, &mut buffer);
                filtering(
                    &buffer,
                    element_length,
                    &mut bip_writer_second,
                    &word_to_filter,
                    &stats_data,
                );
            }
        })?;

    //start get_data_from_socket_send_to_bip_buffer thread
    //Receive data from socket and send to bipbuffer
    let get_data_from_socket_send_to_bip_buffer = thread::Builder::new()
        .name("get_data_from_socket_send_to_bip_buffer".into())
        .spawn(move || loop {
            stats_data2.in_bytes.add(
                socket_reader
                    .receive_data()
                    .chain_err(|| "Error in get_data_from_socket_send_to_bip_buffer")
                    .chain_unwrap() as u64,
            );
            stats_data2.in_packets.add(1);
        })?;

    let bipreader_socketwriter = thread::Builder::new()
        .name("bipreader_socketwriter".into())
        .spawn(move || loop {
            stats_data3.out_bytes.add(
                socket_writer
                    .send_data(&mut bip_reader_second)
                    .chain_err(|| "Error in bipreader_socketwriter thread")
                    .chain_unwrap() as u64,
            );
            stats_data3.out_packets.add(1);
        })?;

    //Joining threads
    get_data_from_socket_send_to_bip_buffer
        .join()
        .expect("Error joining get_data_from_socket_send_to_bip_buffer thread");
    filtering.join().expect("Error joining filtering thread");
    bipreader_socketwriter
        .join()
        .expect("Error joining bipreader_socketwriter thread");
    Ok(())
}
