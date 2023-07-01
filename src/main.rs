#![allow(unused)]

mod semaphore;

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    sync::Arc,
    thread,
    time::Duration,
};

use clap::{arg, command, value_parser, ArgAction, Command};
use color_eyre::{eyre::Context, owo_colors::OwoColorize, Result};
use regex::Regex;
use semaphore::Semaphore;
use walkdir::WalkDir;

fn main() -> Result<()> {
    const IS_NAME_BASED_FLAG: &str = "name-base";
    const SHOW_LINE_NUMBER_FLAG: &str = "line-number";
    const DEPTH_FLAG: &str = "depth";
    const THREAD_COUNT_FLAG: &str = "thread-count";
    const INVERT_MATCH_FLAG: &str = "invert-match";
    const VERBOSE_FLAG: &str = "verbose";

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
        .arg(
            arg!(
                --verbose "Verbose"
            )
            .id(VERBOSE_FLAG), //.default_value("false"),
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
    //println!("is_name_based_search: {}", is_name_based_search);

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
    let is_verbose = match matches.get_one::<bool>(VERBOSE_FLAG) {
        Some(val) => *val,
        None => false,
    };

    if is_verbose {
        println!("variables: {} {}", pattern, path);
    }

    let re =
        Regex::new(&pattern).wrap_err_with(|| format!("Invalid regex pattern: {}", pattern))?;
    let re = Arc::new(re);
    //let re = Regex::new(&pattern).expect("err");

    let semaphore = Arc::new(Semaphore::new(thread_count));
    // semaphore.set_verbose(is_verbose);
    let mut workers = Vec::new();

    for entry in WalkDir::new(&path).max_depth(max_depth) {
        let re = re.clone(); //todo: Arc?
        let entry = entry.wrap_err_with(|| format!("Failed to read entry in {}", &path))?;
        let semaphore = Arc::clone(&semaphore);
        semaphore.wait();
        let pattern = pattern.clone(); //todo: Arc?
        let worker = thread::spawn(move || -> Result<()> {
            let entry_path = entry.path();
            if is_verbose {
                //* println!("path: {:?}", entry_path);
                println!("filename: {:?}", entry_path.file_name());
            }

            let name = match entry_path.file_name() {
                Some(ref val) => match val.to_str() {
                    Some(str) => str,
                    None => "",
                },
                None => "",
            };
            if is_name_based_search {
                if name != "" && (re.is_match(&name) ^ is_invert_match) {
                    println!("{}", entry_path.display());
                }
                semaphore.signal();
                return Ok(());
            }
            if entry_path.is_dir() {
                semaphore.signal();
                return Ok(());
            }

            let file = File::open(&entry_path)
                .wrap_err_with(|| format!("Failed to open file {}", &entry_path.display()))?;
            //let file = File::open(path).expect("Err");
            let reader = BufReader::new(file);

            for (i, line) in reader.lines().enumerate() {
                //todo: warning for error lines instead?
                // let line = match line {
                //     Ok(v) => v,
                //     Err(v) => "".to_string(),
                // };
                let line = line.wrap_err_with(|| {
                    format!("Failed to read line {} in {}", i + 1, &entry_path.display())
                })?;
                //let line = line.expect("err");
                if re.is_match(&line) ^ is_invert_match {
                    if show_line_number {
                        println!(
                            "{}: {}:{}",
                            // entry_path.display(),
                            name,
                            (i + 1).to_string().green(),
                            re.replace_all(&line, |c: &regex::Captures| c
                                .get(0)
                                .unwrap()
                                .as_str()
                                .red()
                                .underline()
                                .to_string())
                        );
                    } else {
                        println!(
                            "{}: {}",
                            entry_path.display(),
                            line
                            // re.replace_all(&line, |c: &regex::Captures| c
                            //     .get(0)
                            //     .unwrap()
                            //     .as_str()
                            //     .red()
                            //     .underline()
                            //     .to_string())
                        );
                    }
                }
            }
            //thread::sleep(Duration::from_secs(1));
            semaphore.signal();
            Ok(())
        });
        workers.push(worker);
    }
    for worker in workers {
        worker.join().unwrap();
    }
    Ok(())
}
