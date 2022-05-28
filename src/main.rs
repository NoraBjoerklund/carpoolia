use std::convert::TryInto;
use std::{fmt::Write, num::ParseIntError};

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:02x}", b).unwrap();
    }
    s
}

#[derive(Copy, Clone, Debug)]
struct GPSData {
    longitude: u32,
    latitude: u32,
    altitude: u16,
    angle: u16,
    satellites: u8,
    speed: u16,
}

#[derive(Copy, Clone, Debug)]
struct AVLData {
    timestamp: u64,
    priority: u8,
    gps_data: GPSData,
    //io_element: IOElements,
}

struct IOElements {
    event_io_id: u8,
    nr_of_io_elements: u8,
    nr_of_1_bytes: u8,
    io_1_bytes: Vec<[u8; 2]>,
    nr_of_2_bytes: u8,
    io_2_bytes: Vec<[u8; 3]>,
    nr_of_4_bytes: u8,
    io_4_bytes: Vec<[u8; 5]>,
    nr_of_8_bytes: u8,
    io_8_bytes: Vec<[u8; 9]>,
}

struct AVLDataArray {
    codec: u8,
    nr_of_data: u8,
    data: Vec<AVLData>,
}

fn read_be_u16(input: &[u8]) -> u16 {
    let (int_bytes, _rest) = input.split_at(std::mem::size_of::<u16>());
    u16::from_be_bytes(int_bytes.try_into().unwrap())
}

fn read_be_u32(input: &[u8]) -> u32 {
    let (int_bytes, _rest) = input.split_at(std::mem::size_of::<u32>());
    u32::from_be_bytes(int_bytes.try_into().unwrap())
}

fn read_be_u64(input: &[u8]) -> u64 {
    let (int_bytes, _rest) = input.split_at(std::mem::size_of::<u64>());
    u64::from_be_bytes(int_bytes.try_into().unwrap())
}

fn read_gps_data(data: &Vec<u8>, pointer: usize) -> GPSData {
    GPSData {
        longitude: read_be_u32(&data[pointer..(pointer + 4)]),
        latitude: read_be_u32(&data[(pointer + 4)..(pointer + 8)]),
        altitude: read_be_u16(&data[(pointer + 8)..(pointer + 10)]),
        angle: read_be_u16(&data[(pointer + 10)..(pointer + 12)]),
        satellites: data[pointer + 12],
        speed: read_be_u16(&data[(pointer + 13)..(pointer + 15)]),
    }
}

fn main() {
    let data = decode_hex(&std::fs::read_to_string("src/data").unwrap()).unwrap();

    let initbytes: &[u8] = &data[0..4];
    let datalength: &[u8] = &data[4..8];
    println!("{:?}", read_be_u32(initbytes));
    println!("{:?}", read_be_u32(datalength));
    let codec_id: u8 = data[8];
    let nr_of_data = data[9];
    println!("codec {}, nr {}", codec_id, nr_of_data);
    let mut avl_data_arr: Vec<AVLData> = vec![];
    let mut pointer = 10;
    for _ in 0..nr_of_data {
        let timestamp_hex = &data[pointer..(pointer + 8)];
        let timestamp = read_be_u64(timestamp_hex);
        let priority = data[pointer + 8];
        println!(
            "Pointer: {}, Timestamp: {:?}, prio: {}",
            pointer, timestamp_hex, priority
        );

        let gps_data = read_gps_data(&data, pointer + 9);
        pointer += 23;

        let _event_io_id = data[pointer + 1];
        let nr_of_io_elements = data[pointer + 2];
        let nr_of_1_bytes = data[pointer + 3];
        let nr_of_2_bytes = data[pointer + 3 + 1 + nr_of_1_bytes as usize * 2];
        let nr_of_4_bytes =
            data[pointer + 3 + 1 + nr_of_1_bytes as usize * 2 + 1 + nr_of_2_bytes as usize * 3];
        let nr_of_8_bytes = data[pointer
            + 3
            + 1
            + nr_of_1_bytes as usize * 2
            + 1
            + nr_of_2_bytes as usize * 3
            + 1
            + nr_of_4_bytes as usize * 5];

        println!(
            "1: {}, 2: {}, 4: {}, 8: {}, total: {}",
            nr_of_1_bytes, nr_of_2_bytes, nr_of_4_bytes, nr_of_8_bytes, nr_of_io_elements
        );
        pointer += 3
            + 1
            + nr_of_1_bytes as usize * 2
            + 1
            + nr_of_2_bytes as usize * 3
            + 1
            + nr_of_4_bytes as usize * 5
            + 1
            + nr_of_8_bytes as usize * 9;

        avl_data_arr.push(AVLData {
            timestamp,
            priority,
            gps_data,
        });
    }
    let check_tot_data = data[pointer];
    if check_tot_data == nr_of_data {
        println!("WIN!");
    } else {
        println!("FAIL: {}, pointer: {}", check_tot_data, pointer);
    }
}
