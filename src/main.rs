#![allow(unused)]

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use clap::{arg, command, value_parser, ArgAction, Command};
use color_eyre::{eyre::Context, Result};
use regex::Regex;
use walkdir::WalkDir;

fn main() -> Result<()> {
    const IS_NAME_BASED_FLAG: &str = "name-base";
    const SHOW_LINE_NUMBER_FLAG: &str = "line-number";
    const DEPTH_FLAG: &str = "depth";
    const THREAD_COUNT_FLAG: &str = "thread-count";
    const INVERT_MATCH_FLAG: &str = "invert-match";

    color_eyre::install();

    let matches = command!() // requires `cargo` feature
        .name("rGrep")
        .about("grep with Rust")
        .arg_required_else_help(true)
        .arg(arg!([pattern] "Regex pattern"))
        .arg(arg!([path] "The path to be searched"))
        .arg(
            arg!(
                -n --"name-base" "Search in file *names* and directories instead of file contents."
            )
            .id(IS_NAME_BASED_FLAG), //.default_value("false"),
        )
        .arg(
            arg!(
                -l --"line-number" "Display the line number of lines found in *content search mode*"
            )
            .id(SHOW_LINE_NUMBER_FLAG),
        )
        .arg(
            arg!(
                -d --depth <number> "Maximum search depth"
            )
            .id(DEPTH_FLAG)
            .default_value("1")
            .value_parser(value_parser!(usize)),
        )
        .arg(
            arg!(
                -t --"thread-count" <count> "Number of threads for search"
            )
            .id(THREAD_COUNT_FLAG)
            .default_value("2")
            .value_parser(value_parser!(usize)),
        )
        .arg(
            arg!(
                -i --"invert-match" "Display non-matching lines or names"
            )
            .id(INVERT_MATCH_FLAG), //.default_value("false"),
        )
        .get_matches();

    let pattern = match matches.get_one::<String>("pattern") {
        Some(ref val) => val.to_string(),
        None => "".to_string(),
    };

    let path = match matches.get_one::<String>("path") {
        Some(ref val) => val.to_string(),
        None => "./".to_string(),
    };

    let is_name_based_search = match matches.get_one::<bool>(IS_NAME_BASED_FLAG) {
        Some(val) => *val,
        None => false,
    };
    println!("is_name_based_search: {}", is_name_based_search);

    let show_line_number = match matches.get_one::<bool>(SHOW_LINE_NUMBER_FLAG) {
        Some(val) => *val,
        None => false,
    };

    let max_depth = match matches.get_one::<usize>(DEPTH_FLAG) {
        Some(val) => *val,
        None => 1,
    };
    let thread_count = match matches.get_one::<usize>(THREAD_COUNT_FLAG) {
        Some(val) => *val,
        None => 2,
    };
    let is_invert_match = match matches.get_one::<bool>(INVERT_MATCH_FLAG) {
        Some(val) => *val,
        None => false,
    };
    println!("variables: {} {}", pattern, path);

    let re =
        Regex::new(&pattern).wrap_err_with(|| format!("Invalid regex pattern: {}", pattern))?;
    //let re = Regex::new(&pattern).expect("err");

    for entry in WalkDir::new(&path).max_depth(max_depth) {
        let entry = entry.wrap_err_with(|| format!("Failed to read entry in {}", &path))?;
        let path = entry.path();
        //* println!("path: {:?}", path);

        //* println!("filename: {:?}", path.file_name());

        if is_name_based_search {
            let name = match path.file_name() {
                Some(ref val) => match val.to_str() {
                    Some(str) => str,
                    None => "",
                },
                None => "",
            };
            if name != "" && (re.is_match(&name) ^ is_invert_match) {
                println!("{}", path.display());
            }
            continue;
        }
        if path.is_dir() {
            continue;
        }

        let file = File::open(&path)
            .wrap_err_with(|| format!("Failed to open file {}", &path.display()))?;
        //let file = File::open(path).expect("Err");
        let reader = BufReader::new(file);

        for (i, line) in reader.lines().enumerate() {
            //todo: warning for error lines instead?
            let line = line.wrap_err_with(|| {
                format!("Failed to read line {} in {}", i + 1, &path.display())
            })?;
            //let line = line.expect("err");

            if re.is_match(&line) ^ is_invert_match {
                if show_line_number {
                    println!("{}: {}:{}", path.display(), i + 1, line);
                } else {
                    println!("{}: {}", path.display(), line);
                }
            }
        }
    }

    Ok(())
}
