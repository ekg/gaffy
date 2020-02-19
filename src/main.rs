#![allow(clippy::too_many_arguments)]

use std::fs::File;
use std::io::{self, prelude::*, BufReader};

extern crate clap;
use clap::{App, Arg};

extern crate rand;
use rand::Rng;

fn gaf_max_id(filename: &str) -> usize {
    let mut max_id = usize::min_value();
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let l = line.unwrap();
        let path = l.split('\t').nth(5).unwrap();
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

fn for_each_line_in_gfa(gfa_filename: &str, line_type: &str, mut callback: impl FnMut(&str)) {
    let file = File::open(gfa_filename).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let l = line.unwrap();
        let curr_type = l.split('\t').nth(0).unwrap();
        if curr_type == line_type {
            callback(&l);
        }
    }
}

struct GfaGraph {
    node_length: Vec<usize>,
    max_id: usize,
}

impl GfaGraph {
    fn new() -> Self {
        GfaGraph {
            node_length: vec![],
            max_id: usize::min_value(),
        }
    }
    fn from_gfa(gfa_filename: &str) -> Self {
        let mut max_id = usize::min_value();
        for_each_line_in_gfa(gfa_filename, "S", |l: &str| {
            let id = l.split('\t').nth(1).unwrap().parse::<usize>().unwrap();
            if id > max_id {
                max_id = id;
            }
        });
        let mut node_length = Vec::<usize>::new();
        node_length.resize(max_id, 0);
        for_each_line_in_gfa(gfa_filename, "S", |l: &str| {
            let id = l.split('\t').nth(1).unwrap().parse::<usize>().unwrap();
            let seq = l.split('\t').nth(2).unwrap();
            node_length[id - 1] = seq.len();
        });
        GfaGraph {
            node_length,
            max_id,
        }
    }
    fn get_node_length(self: &GfaGraph, id: usize) -> usize {
        self.node_length[id - 1]
    }
    fn get_max_id(self: &GfaGraph) -> usize {
        self.max_id
    }
    fn loaded(self: &GfaGraph) -> bool {
        self.max_id != 0
    }
}

fn gaf_nth_longest_read(
    filename: &str,
    keep_n_longest: usize,
    min_length: u64,
    max_length: u64,
) -> u64 {
    let mut v = Vec::new();
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let length = line
            .unwrap()
            .split('\t')
            .nth(1)
            .unwrap()
            .parse::<u64>()
            .unwrap();
        if length >= min_length && length <= max_length {
            v.push(length);
        }
    }
    // sort by decreasing length
    v.sort_by(|a, b| b.partial_cmp(a).unwrap());
    let cutoff = if keep_n_longest > v.len() {
        v.len()
    } else {
        keep_n_longest
    };
    v[cutoff - 1]
}

fn do_matrix(
    filename: &str,
    gfa_filename: &str,
    vectorize: bool,
    binary_out: bool,
    mut max_id: usize,
    min_length: u64,
    max_length: u64,
    trim_read_name: bool,
    group_name: &str,
    keep_n_longest: usize,
    sampling_rate: f64,
) {
    let graph = if !gfa_filename.is_empty() {
        GfaGraph::from_gfa(gfa_filename)
    } else {
        GfaGraph::new()
    };
    max_id = if graph.loaded() {
        graph.get_max_id()
    } else {
        max_id
    };
    if !vectorize && max_id == 0 {
        max_id = gaf_max_id(filename);
    }
    let query_length_threshold = if keep_n_longest > 0 {
        gaf_nth_longest_read(filename, keep_n_longest, min_length, max_length)
    } else {
        u64::min_value()
    };
    let mut rng = rand::thread_rng();
    if group_name != "" {
        print!("group.name\t");
    }
    if vectorize {
        print!("aln.name\tquery.length\tnode.id\trank");
    } else {
        print!("aln.name\tquery.length\tnode.count");
        for x in 1..=max_id {
            print!("\tnode.{}", x);
        }
    }
    println!();
    io::stdout().flush().unwrap();
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        // parse the line
        let l = line.unwrap();
        let mut name = "";
        let mut path = "";
        let mut query_length: u64 = 0;
        for (i, s) in l.split('\t').enumerate() {
            match i {
                0 => {
                    name = if trim_read_name {
                        s.split_ascii_whitespace().nth(0).unwrap()
                    } else {
                        s
                    }
                }
                1 => query_length = s.parse::<u64>().unwrap(),
                5 => path = s,
                _ => {}
            };
        }
        if query_length >= min_length
            && query_length <= max_length
            && query_length >= query_length_threshold
            && (sampling_rate == 1.0 || rng.gen::<f64>() < sampling_rate)
        {
            if vectorize {
                for (j, n) in path.split(|c| c == '<' || c == '>').enumerate() {
                    if !n.is_empty() {
                        if group_name != "" {
                            print!("{}\t", group_name);
                        }
                        println!("{}\t{}\t{}\t{}", name, query_length, n, j);
                    }
                }
            } else {
                let mut v = vec![0; max_id];
                for n in path.split(|c| c == '<' || c == '>') {
                    if !n.is_empty() {
                        let id = n.parse::<usize>().unwrap();
                        v[id - 1] = if binary_out {
                            1
                        } else if graph.loaded() {
                            graph.get_node_length(id)
                        } else {
                            1
                        };
                    }
                }
                if group_name != "" {
                    print!("{}\t", group_name);
                }
                print!("{}", name);
                print!("\t{}", query_length);
                let sum: usize = v.iter().sum();
                print!("\t{}", sum);
                for x in v {
                    print!("\t{}", x);
                }
                println!();
            }
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
        .arg(Arg::with_name("weighted-matrix")
             .short("w")
             .long("weighted-matrix")
             .help("Weight matrix values by GFA node_length."))
        .arg(Arg::with_name("sampling-rate")
             .short("r")
             .long("sampling-rate")
             .takes_value(true)
             .help("Sample selected alignments at this rate [0-1]."))
        .get_matches();

    let filename = matches.value_of("INPUT").unwrap();

    let max_id = usize::min_value();

    let gfa_filename = if matches.is_present("gfa") {
        matches.value_of("gfa").unwrap()
    } else {
        ""
    };

    let keep_n_longest = if matches.is_present("keep-n-longest") {
        matches
            .value_of("keep-n-longest")
            .unwrap()
            .parse::<usize>()
            .unwrap()
    } else {
        0
    };

    let min_length = if matches.is_present("min-length") {
        matches
            .value_of("min-length")
            .unwrap()
            .parse::<u64>()
            .unwrap()
    } else {
        0
    };
    let max_length = if matches.is_present("max-length") {
        matches
            .value_of("max-length")
            .unwrap()
            .parse::<u64>()
            .unwrap()
    } else {
        u64::max_value()
    };
    let sampling_rate = if matches.is_present("sampling-rate") {
        matches
            .value_of("sampling-rate")
            .unwrap()
            .parse::<f64>()
            .unwrap()
    } else {
        1.0
    };

    do_matrix(
        filename,
        gfa_filename,
        matches.is_present("vectorize"),
        !matches.is_present("weighted-matrix"),
        max_id,
        min_length,
        max_length,
        matches.is_present("trim-read-name"),
        matches.value_of("group-name").unwrap_or(""),
        keep_n_longest,
        sampling_rate,
    );

    Ok(())
}
