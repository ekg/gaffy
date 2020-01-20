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

fn gaf_max_id(filename: &str) -> usize {
    let mut max_id = usize::min_value();
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
            }
        }
    }
    max_id
}

fn gfa_max_id(gfa_filename: &str) -> usize {
    let file = File::open(gfa_filename).unwrap();
    let reader = BufReader::new(file);
    let mut max_id = usize::min_value();
    for line in reader.lines() {
        // parse the line
        let l = line.unwrap();
        let linetype = l.split("\t").nth(0).unwrap();
        if linetype == "S" {
            let id = l.split("\t").nth(1).unwrap().parse::<usize>().unwrap();
            if id > max_id {
                max_id = id;
            }
        }
    }
    max_id
}

fn gaf_nth_longest_read(filename: &str,
                        keep_n_longest: usize,
                        min_length: u64,
                        max_length: u64) -> u64 {
    let mut v = Vec::new();
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let length = line.unwrap().split("\t").nth(1).unwrap().parse::<u64>().unwrap();
        if length >= min_length && length <= max_length {
            v.push(length);
        }
    }
    // sort by decreasing length
    v.sort_by(|a, b| b.partial_cmp(a).unwrap());
    let cutoff = if keep_n_longest > v.len() { v.len() } else { keep_n_longest };
    v[cutoff-1]
}

fn do_matrix(filename: &str,
             mut max_id: usize,
             min_length: u64,
             max_length: u64,
             trim_read_name: bool,
             group_name: &str,
             keep_n_longest: usize) {
    if max_id == 0 {
        max_id = gaf_max_id(filename);
    }
    let mut query_length_threshold = u64::min_value();
    if keep_n_longest > 0 {
        query_length_threshold = gaf_nth_longest_read(filename, keep_n_longest, min_length, max_length);
    }
    if group_name != "" {
        print!("group.name\t");
    }
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
        if query_length >= min_length
            && query_length <= max_length
            && query_length >= query_length_threshold {
            if group_name != "" {
                print!("{}\t", group_name);
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
        .arg(Arg::with_name("gfa")
             .short("g")
             .long("gfa")
             .takes_value(true)
             .help("Input GFA file to which the GAF was mapped."))
        .arg(Arg::with_name("vectorize")
             .short("v")
             .long("vectorize")
             .help("Write a tabular representation of the alignments (one record per alignment node traversal)"))
        .arg(Arg::with_name("matrix")
             .short("m")
             .long("matrix")
             .help("Write a binary matrix representing coverage across the graph (two passes over input file)."))
        .arg(Arg::with_name("trim-read-name")
             .short("t")
             .long("trim-read-name")
             .help("Trim the read name at the first whitespace"))
        .arg(Arg::with_name("group-name")
             .short("n")
             .long("group-name")
             .takes_value(true)
             .help("Add a group name field to each record in the matrix or vector output, to help when merging outputs."))
        .arg(Arg::with_name("keep-n-longest")
             .short("k")
             .long("keep-n-longest")
             .takes_value(true)
             .help("Keep the longest N reads."))
        .arg(Arg::with_name("max-length")
             .short("M")
             .long("max-length")
             .takes_value(true)
             .help("Keep reads shorter than this length (before keep-n-longest calculations)."))
        .arg(Arg::with_name("min-length")
             .short("L")
             .long("min-length")
             .takes_value(true)
             .help("Keep reads longer than this length (before keep-n-longest calculations)."))
        .get_matches();
    let filename = matches.value_of("INPUT").unwrap();
    if matches.is_present("vectorize") {
        do_vectorize(filename, matches.is_present("trim-read-name"));
    } else if matches.is_present("matrix") {
        let max_id = if matches.is_present("gfa") {
            gfa_max_id(matches.value_of("gfa").unwrap())
        } else {
            usize::min_value()
        };
        let keep_n_longest = if matches.is_present("keep-n-longest") {
            matches.value_of("keep-n-longest").unwrap().parse::<usize>().unwrap()
        } else {
            0
        };
        let min_length = if matches.is_present("min-length") {
            matches.value_of("min-length").unwrap().parse::<u64>().unwrap()
        } else {
            0
        };
        let max_length = if matches.is_present("max-length") {
            matches.value_of("max-length").unwrap().parse::<u64>().unwrap()
        } else {
            u64::max_value()
        };
        do_matrix(filename,
                  max_id,
                  min_length,
                  max_length,
                  matches.is_present("trim-read-name"),
                  matches.value_of("group-name").unwrap_or(""),
                  keep_n_longest);
    }
    Ok(())
}
