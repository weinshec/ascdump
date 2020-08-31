use std::fs::File;

extern crate clap;
use clap::{App, Arg};

use ascdump::AscParser;

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

    let input_file = File::open(args.value_of("INPUT").unwrap()).expect("TODO: remove this unwrap");
    let parser = AscParser::new(input_file);

    for frame in parser.filter(|frame| frame.id == 2000) {
        println!("{:?}", frame);
    }
}
