use hound;
use std::i16;
use std::fs::File;
use std::io::prelude::*;
use itertools::izip;

const BITS_PER_BYTE: u32 = 2;
const BUFFER_SIZE: usize = 4096;

fn generate_bit_mask(n: u32) -> u32
{
    (2 as u32).pow(n) - 1
}

fn steganography_file(file: &mut File) -> (Vec<u8>, usize) {
    let mut bytes : Vec<u8> = Vec::new();

    let mut buf = Vec::new(); 
    let readed = file.read_to_end(&mut buf).expect("Error reading steganography file");
    
    for i in 0..readed {
        //println!("Byte of file: {}", buf[i]);
        let mask = generate_bit_mask(BITS_PER_BYTE);

        let mut j = 0;
        let mut temp_byte = buf[i];

        while j < 8 {
            j += BITS_PER_BYTE;
            let b = temp_byte & mask as u8;
            temp_byte = temp_byte >> BITS_PER_BYTE;
            bytes.push(b);
        }
    }

    (bytes, readed)
}

fn encrypt() {
    let mut reader = hound::WavReader::open("audio/GET_IT_BY_YOUR_HANDS.wav")
    .expect("Error opening input audio file.");
    let mut writer = hound::WavWriter::create("audio/GET_IT_BY_YOUR_HANDS_cropped.wav", reader.spec())
        .expect("Error creating output audio file.");

    let mut file = File::open("imgs/test.jpeg")
        .expect("Error opening stealth file.");

    let samples = reader.samples::<i16>();
    let (steganography, _) = steganography_file(&mut file);
    let mask = generate_bit_mask(BITS_PER_BYTE);

    println!("Samples length {}", samples.len());

    for (orig_sample, stega_byte) in izip!(samples, steganography) {
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
        
        let c = (sample as u16 & mask as u16);
        let f = j;
        let b = c << f;
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
        encrypt()
    } else {
        let bytes = decrypt();
        let mut file = File::create("./decrypted.jpg").expect("Error creating decrypted file.");
        file.write_all(bytes.as_slice()).expect("Error writing in the decrypted file.");
    }

    
}
