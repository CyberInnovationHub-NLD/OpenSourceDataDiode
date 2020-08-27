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

use framework_constants::*;
use spsc_bip_buffer::BipBufferReader;
use spsc_bip_buffer::BipBufferWriter;

///This function is used to write to the bip_buffer using the supplied writer.
/// # Arguments
/// * `writer` - The bipBufferWriter used to write to the bip_buffer.
/// * `buffer` - The buffer that should be written to the bip_buffer.
pub fn write_to_bip_buffer(writer: &mut BipBufferWriter, buffer: &[u8]) {
    let mut reservation = writer.spin_reserve(buffer.len() + BIP_BUFFER_LEN_FIELD_LEN);
    let byte_len = buffer.len().to_le_bytes();
    reservation[0..BIP_BUFFER_LEN_FIELD_LEN].copy_from_slice(&byte_len);
    reservation[BIP_BUFFER_LEN_FIELD_LEN..buffer.len() + BIP_BUFFER_LEN_FIELD_LEN]
        .copy_from_slice(&buffer);
    reservation.send();
}

///This function is used to read from the bip_buffer using the supplied reader.
/// # Arguments
/// * `reader` - The bipBufferReader used to read from the bip_buffer.
/// * `buffer` - The buffer to be filled with data from the bip_buffer.
/// # Returns
/// * `usize` - The amount of bytes read from the bip_buffer.
pub fn read_from_bip_buffer(reader: &mut BipBufferReader, buffer: &mut [u8]) -> usize {
    let element_length = get_element_length(reader);
    //read data from the buffer
    wait_for_data(reader, element_length);
    let incoming = reader.valid();
    //copy incoming data (excluding length field) into buffer
    buffer[..element_length].copy_from_slice(&incoming[..element_length]);
    //mark incoming data as consumed
    reader.consume(element_length);
    element_length
}

///Function used to get the element length field from the bip_buffer.
///The element length is consumed from the buffer by calling this function.
/// # Arguments
/// * `reader` - The bipBufferReader used to read the element length.
/// # Returns
/// * `usize` - The length of the read element in bytes.
pub fn get_element_length(reader: &mut BipBufferReader) -> usize {
    wait_for_data(reader, BIP_BUFFER_LEN_FIELD_LEN);
    let length_buffer = &mut reader.valid()[..BIP_BUFFER_LEN_FIELD_LEN];
    let mut length_bytes_fixed = [0; BIP_BUFFER_LEN_FIELD_LEN];
    length_bytes_fixed.copy_from_slice(&length_buffer[0..BIP_BUFFER_LEN_FIELD_LEN]);
    let element_length = usize::from_le_bytes(length_bytes_fixed);
    reader.consume(BIP_BUFFER_LEN_FIELD_LEN);
    element_length
}

///Wait for the given amount of bytes to be available for reading in the bip_buffer.
/// # Arguments
/// * `reader` - The bipBufferReader used to read from the bip_buffer.
/// * `bytes` - The amount of available bytes to wait for
pub fn wait_for_data(reader: &mut BipBufferReader, bytes: usize) {
    let mut spin_count = 0;
    while reader.valid().len() < (bytes) {
        if spin_count < 100_000 {
            std::sync::atomic::spin_loop_hint();
            spin_count += 1;
        } else {
            std::thread::sleep(std::time::Duration::from_millis(100));
            spin_count = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::get_element_length;
    use crate::wait_for_data;
    use crate::write_to_bip_buffer;
    use framework_constants::*;
    use spsc_bip_buffer::bip_buffer_with_len;
    use spsc_bip_buffer::BipBufferReader;
    #[test]
    ///Is used to test reading and writing of multiple buffers to a bip_buffer.
    fn write_read_bip_buffer() {
        let mut test_buffers = create_test_buffers();
        let (mut writer, mut reader) = bip_buffer_with_len(MAX_BIP_BUFFER_MESSAGE_SIZE * 10);
        for i in 0..test_buffers.len() {
            write_to_bip_buffer(&mut writer, &test_buffers[i]);
            assert_on_byte_array(&mut reader, &mut test_buffers[i]);
        }
    }

    ///asserts if the given buffer equals the buffer read from the bip_buffer.
    fn assert_on_byte_array(receiver_reader: &mut BipBufferReader, send_buffer: &[u8]) {
        let element_length = get_element_length(receiver_reader);
        //assert_eq!(send_buffer.len(), element_length);
        wait_for_data(receiver_reader, element_length);
        let bip_buffer = receiver_reader.valid();
        let received_message = &bip_buffer[..element_length];
        //compare the sent buffer to the buffer that was received.
        assert_eq!(&send_buffer[..], &received_message[..]);
        assert_eq!(send_buffer.len(), received_message.len());
        receiver_reader.consume(element_length);
    }

    ///returns a vector of multiple Vec<u8>
    fn create_test_buffers() -> Vec<Vec<u8>> {
        let mut buffer1 = vec![0; MAX_BUFFER_SIZE_BYTES];
        for i in 0..buffer1.len() {
            buffer1[i] = i as u8;
        }
        let mut buffer2 = vec![0; 2];
        for i in 0..buffer2.len() {
            buffer2[i] = i as u8;
        }
        let mut buffer3 = vec![0; 1000];
        for i in 0..buffer3.len() {
            buffer3[i] = i as u8;
        }
        let mut buffer4 = vec![0; 65500];
        for i in 0..buffer4.len() {
            buffer4[i] = i as u8;
        }
        let mut buffer5 = vec![0; 2];
        for i in 0..buffer5.len() {
            buffer5[i] = i as u8;
        }
        vec![buffer1, buffer2, buffer3, buffer4, buffer5]
    }
}
