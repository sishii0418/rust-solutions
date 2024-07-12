use std::error::Error;
use clap::{App, Arg};
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("catr")
        .version("0.1.0")
        .author("Shunsuke Ishii")
        .about("Rust cat")
        .arg(
            Arg::with_name("files")
                .value_name("FILES")
                .help("Input file(s)")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("number")
                .short("n")
                .long("number")
                .help("Number lines")
                .takes_value(false)
                .conflicts_with("number_nonblank_lines"),
        )
        .arg(
            Arg::with_name("number_nonblank")
                .short("b")
                .long("number-nonblank")
                .help("Number non-blank lines")
                .takes_value(false),
        )
        .get_matches();

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        number_lines: matches.is_present("number"),
        number_nonblank_lines: matches.is_present("number_nonblank"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    for filename in config.files {
        match open(&filename) {
            Err(err) => eprintln!("Failed to open {}: {}", filename, err),
            Ok(input) => {
                // Collect lines into a vector to avoid consuming `input` multiple times
                let lines: Vec<Result<String, std::io::Error>> = input.lines().collect();

                if config.number_lines {
                    for (index, line) in lines.iter().enumerate() {
                        println!("{}{}", format!("{:>6}\t", index + 1), line.as_ref().unwrap());
                    }
                } else if config.number_nonblank_lines {
                    let mut index = 1;
                    for line in lines.iter() {
                        if !line.as_ref().unwrap().is_empty() {
                            println!("{}{}", format!("{:>6}\t", index), line.as_ref().unwrap());
                            index += 1;
                        } else {
                            println!();
                        }
                    }
                } else {
                    for line in lines.iter() {
                        println!("{}", line.as_ref().unwrap());
                    }
                }
            }
        }
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _   => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}