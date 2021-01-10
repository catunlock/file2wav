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

fn encrypt(input_file: &str, hide_file: &str, output_file: &str) {
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

    for (orig_sample, stega_byte) in izip!(samples, stega_data) {
        let orig_sample = orig_sample.expect("Error reading audio file");

        let result = (orig_sample & !mask as i16 ) |  stega_byte as i16;
        //println!("Audio Value: {} -> Crop: {}", sample, crop);
        writer.write_sample(result).unwrap();

    }

    println!("Samples writed {}", writer.len());

    writer.finalize().unwrap();
}

fn decrypt() -> Vec<u8> {
    let mut bytes : Vec<u8> = Vec::new();

    let mut reader = hound::WavReader::open("audio/GET_IT_BY_YOUR_HANDS_cropped.wav")
        .expect("Error opening input audio file.");

    let mut mask = generate_bit_mask(BITS_PER_BYTE);
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

    let command = std::env::args().nth(1).expect("no command given, use [encrypt, decrypt]");

    if command == "encrypt" {
        encrypt("audio/GET_IT_BY_YOUR_HANDS.wav", "imgs/bici.jpg" ,"audio/GET_IT_BY_YOUR_HANDS_cropped.wav")
    } else {
        let bytes = decrypt();
        let mut file = File::create("./decrypted.jpg").expect("Error creating decrypted file.");
        file.write_all(bytes.as_slice()).expect("Error writing in the decrypted file.");
    }

    
}
