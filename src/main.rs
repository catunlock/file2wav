use hound;
use std::i16;
use std::fs::File;
use std::io::prelude::*;
use clap::{Arg, App, SubCommand};

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

fn insert(input_wav: &str, hide_file: &str, output_wav: &str) {
    let mut reader = hound::WavReader::open(input_wav)
    .expect("Error opening input audio file.");
    let mut writer = hound::WavWriter::create(output_wav, reader.spec())
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

fn recover(input_wav: &str) -> Vec<u8> {
    let mut bytes : Vec<u8> = Vec::new();

    let mut reader = hound::WavReader::open(input_wav)
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
    let matches = App::new("file2wav")
        .version("0.1.0")
        .author("Alberto López Sánchez <alberto.lopez.s@outlook.es>")
        .about("insert any file into a WAV audio file")

        .arg(Arg::with_name("input_wav")
            .help("Sets the input audio WAV file to use")
            .required(true)
            .index(1))

        .arg(Arg::with_name("Bits Per Sample")
            .short("bps")
            .long("bits-per-sample")
            .value_name("bps")
            .help("Sets a custom config file")
            .takes_value(true)
            .default_value("2"))
        
        .arg(Arg::with_name("Skip")
            .short("s")
            .long("skip")
            .value_name("skip")
            .help("How many bits of the original file are inserted in the LSB (Less significant bits) of the audio samples. max 8.")
            .takes_value(true)
            .default_value("0"))
        
        .subcommand(SubCommand::with_name("insert")
            .about("insert file in WAV audio file.")
            .arg(Arg::with_name("hide_file")
                .help("File to insert in the WAV file")
                .required(true)
                )
            .arg(Arg::with_name("output_wav")
                .help("Name of the output audio WAV file")
                .required(true)
                ))
                    
        .subcommand(SubCommand::with_name("recover")
            .about("recover a file previously inserted in a WAV audio file.")
            .arg(Arg::with_name("output_file")
                .help("Name of the output recovered file")
                .required(true)
                ))
        
        .get_matches();

    let input_wav = matches.value_of("input_wav").unwrap();
    
    if let Some(matches) = matches.subcommand_matches("insert") {
        let hide_file = matches.value_of("hide_file").unwrap();
        let output_wav = matches.value_of("output_wav").unwrap();

        insert(input_wav, hide_file ,output_wav)
    }
    
    if let Some(matches) = matches.subcommand_matches("recover") {
        let output_file = matches.value_of("output_file").unwrap();

        let bytes = recover(input_wav);
        let mut file = File::create(output_file).expect("Error creating recovered file.");
        file.write_all(bytes.as_slice()).expect("Error writing the recovered file.");
    }    
}
