//use std::slice;
//use std::str;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};

extern crate clap;
use clap::{Arg, App};

fn do_vectorize(reader: BufReader<File>, trim_read_name: bool) {
    println!("name\tquery.length\tnode.id\trank");
    for line in reader.lines() {
        // parse the line
        let l = line.unwrap();
        //println!("{}", l);
        let mut i = 0;
        let mut name = "";
        let mut path = "";
        let mut query_length: u64 = 0;
        for s in l.split("\t") {
            i = i + 1;
            match i {
                1 => name = if trim_read_name { s.split_ascii_whitespace().nth(0).unwrap() } else { s },
                2 => query_length = s.parse::<u64>().unwrap(),
                6 => path = s,
                _ => { },
            };
        }
        i = 0;
        for n in path.split(|c| c == '<' || c == '>') {
            if !n.is_empty() {
                i += 1;
                println!("{}\t{}\t{}\t{}", name, query_length, n, i);
            }
        }
    }
}

fn main() -> io::Result<()> {
    let matches = App::new("gaffy")
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
        .arg(Arg::with_name("trim-read-name")
             .short("t")
             .long("trim-read-name")
             .help("Trim the read name at the first whitespace"))
        .get_matches();
    let filename = matches.value_of("INPUT").unwrap();
    //println!("{}", filename);
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    if matches.is_present("vectorize") {
        do_vectorize(reader, matches.is_present("trim-read-name"));
    }
    Ok(())
}
