use hound;
use std::{i16, usize};
use std::fs::File;
use std::io::prelude::*;
use clap::{Arg, App, SubCommand};

fn generate_bit_mask(n: u8) -> u32
{
    (2 as u32).pow(n as u32) - 1
}

fn divide_bytes(file: &mut File, bits_per_byte: u8) -> Vec<u8> {
    let mut bytes : Vec<u8> = Vec::new();

    let mut buf = Vec::new(); 
    let readed = file.read_to_end(&mut buf).expect("Error reading insert file.");
    let mask = generate_bit_mask(bits_per_byte) as u8;

    for i in 0..readed {
        let mut j = 0;
        let mut byte = buf[i];

        while j < 8 {
            j += bits_per_byte;
            bytes.push(byte & mask);
            if bits_per_byte < 8 {
                // Avoid the overflow error.
                byte = byte >> bits_per_byte;
            }
            
        }
    }

    bytes
}

fn insert(input_wav: &str, hide_file: &str, output_wav: &str, bits_per_byte: u8, skip: usize) -> usize {
    let mut reader = hound::WavReader::open(input_wav)
    .expect("Error opening input audio file.");

    println!("{:?}", reader.spec());

    let mut writer = hound::WavWriter::create(output_wav, reader.spec())
        .expect("Error creating output audio file.");

    let mut file = File::open(hide_file)
        .expect("Error opening stealth file.");

    let samples = reader.samples::<i16>();
    let stega_data : Vec<u8> = divide_bytes(&mut file, bits_per_byte);
    let mask = generate_bit_mask(bits_per_byte);

    println!("Samples length {}", samples.len());
    let audio_samples: Vec<i16> = samples.into_iter()
        .map(|a| a.expect("Error reading audio sample"))
        .collect();

    for (i, data) in stega_data.iter().enumerate() {
        let orig_sample = audio_samples[i % audio_samples.len()];
        let sample_inserted = (orig_sample & !mask as i16 ) |  *data as i16;

        writer.write_sample(sample_inserted).expect("Error writing output audio file");
    }

    // Finish writing the original song
    if stega_data.len() < audio_samples.len() {
        for j in stega_data.len()..audio_samples.len() {
            writer.write_sample(audio_samples[j]).expect("Error writing output audio file");
        }
    }

    if writer.len() % 2 != 0 {
        println!("Warning: Unpair resulting number of samples, fixing duplicating the last one.");
        writer.write_sample(audio_samples[audio_samples.len() -1]);
    }

    writer.finalize().unwrap();

    stega_data.len()
}

fn recover(input_wav: &str, bits_per_byte: u8, skip: usize, samples_used: usize) -> Vec<u8> {
    let mut bytes : Vec<u8> = Vec::new();

    let mut reader = hound::WavReader::open(input_wav)
        .expect("Error opening input audio file.");

    println!("{:?}", reader.spec());

    let mask = generate_bit_mask(bits_per_byte);
    let samples = reader.samples::<i16>();

    let mut j = 0;
    let mut temp_byte: u8 = 0;

    // TODO: Change for.
    for sample in samples {
        let sample = sample.expect("Error reading audio file");
        
        let c = sample as u16 & mask as u16;
        let b = c << j;
        temp_byte = temp_byte | (b as u8);

        j += bits_per_byte;

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

        .arg(Arg::with_name("bits_per_byte")
            .short("bps")
            .long("bits-per-sample")
            .value_name("bps")
            .help("Sets a custom config file")
            .takes_value(true)
            .default_value("2"))
        
        .arg(Arg::with_name("skip")
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
            .arg(Arg::with_name("samples_used")
                .help("Number of samples used in the audio by the inserted file.")
                .required(true))
            .arg(Arg::with_name("output_file")
                .help("Name of the output recovered file")
                .required(true)))

        
        .get_matches();

    let input_wav = matches.value_of("input_wav").unwrap();
    let bits_per_byte = matches.value_of("bits_per_byte").unwrap();
    let skip = matches.value_of("skip").unwrap();

    let bits_per_byte = bits_per_byte.parse::<u8>().expect("Invalid bits per byte argument");
    if ! vec![1,2,4,8].contains(&bits_per_byte) {
        panic!("Bits per byte must be: 1, 2, 4, 8");
    }

    let skip = skip.parse::<usize>().expect("Invalid skip argument");
    
    if let Some(matches) = matches.subcommand_matches("insert") {
        let hide_file = matches.value_of("hide_file").unwrap();
        let output_wav = matches.value_of("output_wav").unwrap();

        let samples_used = insert(input_wav, hide_file ,output_wav, bits_per_byte, skip);
        println!("Samples used: {}", samples_used);
    }
    
    if let Some(matches) = matches.subcommand_matches("recover") {
        let output_file = matches.value_of("output_file").unwrap();
        let samples_used = matches.value_of("samples_used").unwrap();
        let samples_used = samples_used.parse::<usize>().expect("Invalid samples used parameter.");

        let bytes = recover(input_wav, bits_per_byte, skip, samples_used);
        let mut file = File::create(output_file).expect("Error creating recovered file.");
        file.write_all(bytes.as_slice()).expect("Error writing the recovered file.");
    }    
}
