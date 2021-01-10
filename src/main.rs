use hound;
use std::i16;
use std::fs::File;
use std::io::prelude::*;
use itertools::izip;

const BITS_PER_BYTE: u8 = 2;

fn generate_bit_mask(n: u8) -> u32
{
    (2 as u32).pow(n as u32) - 1
}

fn divide_bytes(file: &mut File, n_bits: u8) -> Vec<u8> {
    let mut bytes : Vec<u8> = Vec::new();

    let mut buf = Vec::new(); 
    let readed = file.read_to_end(&mut buf).expect("Error reading steganography file");
    let mask = generate_bit_mask(n_bits) as u8;

    for i in 0..readed {
        let mut j = 0;
        let mut byte = buf[i];

        while j < 8 {
            j += n_bits;
            bytes.push(byte & mask);
            byte = byte >> n_bits;
        }
    }

    bytes
}

fn insert(input_file: &str, hide_file: &str, output_file: &str) {
    let mut reader = hound::WavReader::open(input_file)
    .expect("Error opening input audio file.");
    let mut writer = hound::WavWriter::create(output_file, reader.spec())
        .expect("Error creating output audio file.");

    let mut file = File::open(hide_file)
        .expect("Error opening stealth file.");

    let samples = reader.samples::<i16>();
    let stega_data : Vec<u8> = divide_bytes(&mut file, BITS_PER_BYTE);
    let mask = generate_bit_mask(BITS_PER_BYTE);

    println!("Samples length {}", samples.len());
    let audio_samples: Vec<i16> = samples.into_iter()
        .map(|a| a.expect("Error reading audio sample"))
        .collect();

    for (i, data) in stega_data.iter().enumerate() {
        let orig_sample = audio_samples[i % audio_samples.len()];
        let sample_inserted = (orig_sample & !mask as i16 ) |  *data as i16;

        writer.write_sample(sample_inserted).expect("Error writing output audio file");
    }

    println!("Samples writed {}", writer.len());

    writer.finalize().unwrap();
}

fn recover(input_audio: &str) -> Vec<u8> {
    let mut bytes : Vec<u8> = Vec::new();

    let mut reader = hound::WavReader::open(input_audio)
        .expect("Error opening input audio file.");

    let mask = generate_bit_mask(BITS_PER_BYTE);
    let samples = reader.samples::<i16>();

    let mut j = 0;
    let mut temp_byte: u8 = 0;

    for sample in samples {
        let sample = sample.expect("Error reading audio file");
        
        let c = sample as u16 & mask as u16;
        let b = c << j;
        temp_byte = temp_byte | (b as u8);

        j += BITS_PER_BYTE;

        if j >= 8 {
            bytes.push(temp_byte);
            j = 0;
            temp_byte = 0;
        }
    }

    bytes
}

fn main() {
    println!("file2wav");

    let command = std::env::args().nth(1).expect("no command given, use [insert, recover]");

    if command == "insert" {
        insert("audio/kauwela.wav", "imgs/test_bici.jpg" ,"kauwela_inserted.wav")
    } else if command == "recover" {
        let bytes = recover("kauwela_inserted.wav");
        let mut file = File::create("./recovered.jpg").expect("Error creating recovered file.");
        file.write_all(bytes.as_slice()).expect("Error writing in the recovered file.");
    }

    
}
