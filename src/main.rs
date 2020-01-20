use std::fs::File;
use std::io::{self, prelude::*, BufReader};

extern crate clap;
use clap::{Arg, App};

fn do_vectorize(filename: &str, trim_read_name: bool) {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
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

fn gaf_max_min_id(filename: &str) -> (usize, usize) {
    let mut max_id = usize::min_value();
    let mut min_id = usize::max_value();
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let l = line.unwrap();
        let path = l.split("\t").nth(5).unwrap();
        for n in path.split(|c| c == '<' || c == '>') {
            if !n.is_empty() {
                let id = n.parse::<usize>().unwrap();
                if id > max_id {
                    max_id = id;
                }
                if id < min_id {
                    min_id = id;
                }
            }
        }
    }
    (min_id, max_id)
}

fn do_matrix(filename: &str, trim_read_name: bool) {
    let (_min_id, max_id) = gaf_max_min_id(filename);
    print!("aln.name\tquery.length\tnode.count");
    for x in 1..=max_id {
        print!("\tnode.{}", x);
    }
    print!("\n");
    io::stdout().flush().unwrap();
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        // parse the line
        let l = line.unwrap();
        let mut name = "";
        let mut path = "";
        let mut query_length: u64 = 0;
        for (i,s) in l.split("\t").enumerate() {
            match i {
                0 => name = if trim_read_name { s.split_ascii_whitespace().nth(0).unwrap() } else { s },
                1 => query_length = s.parse::<u64>().unwrap(),
                5 => path = s,
                _ => { },
            };
        }
        let mut v = vec![0; max_id];
        for n in path.split(|c| c == '<' || c == '>') {
            if !n.is_empty() {
                let id = n.parse::<usize>().unwrap();
                v[id-1] = 1;
            }
        }
        print!("{}", name);
        print!("\t{}", query_length);
        let sum: u64 = v.iter().sum();
        print!("\t{}", sum);
        for x in v {
            print!("\t{}", x);
        }
        print!("\n");
        io::stdout().flush().unwrap(); // maybe not necessary
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
        .arg(Arg::with_name("matrix")
             .short("m")
             .long("matrix")
             .help("Write a binary matrix representing coverage across the graph (two passes over input file)."))
        .get_matches();
    let filename = matches.value_of("INPUT").unwrap();
    //println!("{}", filename);
    if matches.is_present("vectorize") {
        do_vectorize(filename, matches.is_present("trim-read-name"));
    } else if matches.is_present("matrix") {
        do_matrix(filename, matches.is_present("trim-read-name"));
    }
    Ok(())
}
