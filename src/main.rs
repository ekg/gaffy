extern crate clap;

use clap::{Arg, App};
use std::fs::File;
use std::io::{self, prelude::*, BufReader};

fn main() -> io::Result<()> {
    let matches = App::new("gaffer")
        .version("0.1.0")
        .author("Erik Garrison <erik.garrison@gmail.com>")
        .about("Manipulate GAF (graph alignment format) files")
        .arg(Arg::with_name("INPUT")
             .required(true)
             .takes_value(true)
             .index(1)
             .help("input GAF file"))
        .arg(Arg::with_name("vectorize")
             .short("v")
             .long("vectorize")
             .help("Write a tabular representation of the alignments (one record per alignment node traversal)"))
        .get_matches();
    let filename = matches.value_of("INPUT").unwrap();
    println!("{}", filename);
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        // parse the line
        println!("{}", line?);
    }
    Ok(())
}
