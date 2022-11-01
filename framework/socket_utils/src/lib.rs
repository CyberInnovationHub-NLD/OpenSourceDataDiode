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

pub mod buffered_socket_reader;
pub mod buffered_socket_writer;
pub mod errors;
pub mod socket_reader;
pub mod socket_writer;

pub const SOCKET_PATH_INGRESS: &str = "/tmp/handler_to_transport";
pub const SOCKET_PATH_EGRESS: &str = "/tmp/transport_to_handler";

#[cfg(test)]
mod test {
    mod regular {
        use crate::socket_reader::SocketReader;
        use crate::socket_writer::SocketWriter;
        use framework_constants::*;
        #[test]
        fn read_write_single_element_test() {
            let path = "/tmp/read_write_single_element_regular";
            //add data to bip_buffer
            let buffer = vec![2; MAX_BUFFER_SIZE_BYTES];
            //start socket writer and socket reader
            let mut thread_buffer_clone = buffer.clone();
            std::thread::spawn(move || {
                let mut socket_writer =
                    SocketWriter::start_listening(path).expect("cant create socket writer");
                socket_writer
                    .send_data(&mut thread_buffer_clone)
                    .expect("Can't send data");
                std::thread::sleep(std::time::Duration::from_secs(2));
                socket_writer.stop();
            });

            let mut socket_reader = SocketReader::new(path).expect("Cant create socket reader");
            let mut received_buffer = [0; MAX_BUFFER_SIZE_BYTES];
            socket_reader
                .receive_data(&mut received_buffer)
                .expect("Can't receive data");
            socket_reader.stop().expect("can't stop socket reader");

            //assert on data
            assert_eq!(&buffer[..], &received_buffer[..]);
            assert_eq!(&buffer[..].len(), &received_buffer[..].len());
        }
    }
    mod buffered {
        use crate::buffered_socket_reader::BufferedSocketReader;
        use crate::buffered_socket_writer::BufferedSocketWriter;
        use bip_utils::read_from_bip_buffer;
        use bip_utils::write_to_bip_buffer;
        use framework_constants::*;
        #[test]
        fn read_write_single_element_test() {
            let path = "/tmp/read_write_single_element_buffered";

            let (mut in_writer, mut in_reader) =
                spsc_bip_buffer::bip_buffer_with_len(MAX_BIP_BUFFER_MESSAGE_SIZE * 10);
            //add data to bip_buffer
            let buffer = vec![2; MAX_BUFFER_SIZE_BYTES];
            write_to_bip_buffer(&mut in_writer, &buffer);
            //start socket writer and socket reader
            std::thread::spawn(move || {
                let mut socket_writer = BufferedSocketWriter::start_listening(path)
                    .expect("can't create socket writer");
                socket_writer
                    .send_data(&mut in_reader)
                    .expect("Cant send data");
                std::thread::sleep(std::time::Duration::from_secs(2));
                socket_writer.stop();
            });

            let (out_writer, mut out_reader) =
                spsc_bip_buffer::bip_buffer_with_len(MAX_BIP_BUFFER_MESSAGE_SIZE * 10);

            let mut socket_reader =
                BufferedSocketReader::new(path, out_writer).expect("Can't create socket reader");
            socket_reader.receive_data().expect("can't receive data");
            socket_reader.stop().expect("can't stop socket reader");

            //assert on data
            let mut received_buffer = [0; MAX_BUFFER_SIZE_BYTES];
            read_from_bip_buffer(&mut out_reader, &mut received_buffer);
            assert_eq!(&buffer[..], &received_buffer[..]);
            assert_eq!(&buffer[..].len(), &received_buffer[..].len());
        }
    }
}
