use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::str::FromStr;

extern crate clap;
use clap::{App, Arg};

use ascdump::CanFrame;

fn main() {
    let args = App::new("ascdump")
        .version("0.1")
        .author("Christoph Weinsheimer <christoph.weinsheimer@esrlabs.com>")
        .about("Does awesome things")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input asc file to use")
                .required(true)
                .index(1),
        )
        .get_matches();

    let input_file_path = args.value_of("INPUT").expect("save to call");
    let input_file = File::open(input_file_path).expect("TODO: remove this unwrap");
    let input_reader = BufReader::new(input_file);

    for line in input_reader.lines() {
        if let Ok(line) = line {
            if let Ok(frame) = CanFrame::from_str(&line) {
                println!("{:?}", frame);
            }
        }
    }
}
