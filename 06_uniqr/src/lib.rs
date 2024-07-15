use clap::{App, Arg};
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    in_file: String,
    out_file: Option<String>,
    count: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("uniqr")
        .version("0.1.0")
        .author("Shunsuke Ishii")
        .about("Rust uniq")
        .arg(
            Arg::with_name("in_file")
                .value_name("IN_FILE")
                .help("Input file")
                .required(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("out_file")
                .value_name("OUT_FILE")
                .help("Output file"),
        )
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .help("Show counts")
                .takes_value(false),
        )
        .get_matches();

    Ok(Config {
        in_file: matches.value_of_lossy("in_file").unwrap().to_string(),
        out_file: matches.value_of_lossy("out_file").map(|s| s.to_string()),
        count: matches.is_present("count"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let mut file = open(&config.in_file)
        .map_err(|e| format!("{}: {}", config.in_file, e))?;
    let mut line = String::new();
    let mut count: u64 = 0;
    let mut last_line = String::new();
    let mut beginning = true;
    let mut out_file: Option<File> = None;
    if config.out_file != None {
        out_file = match File::create(config.out_file.as_ref().unwrap()) {
                Ok(out_file) => Some(out_file),
                Err(e) => panic!("Tried to create file but there was a problem: {:?}", e),
            };
    }
    let mut to_be_written = String::new();
    loop {
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            if config.out_file.as_ref() != None {
                to_be_written += &format!("{}{}",
                    if config.count && count > 0 { format!("{:>4} ", count) } else { "".to_string() },
                    last_line);
                let _ = out_file.as_ref().unwrap().write_all(to_be_written.as_bytes());
            } else {
                print!("{}{}",
                    if config.count && count > 0 { format!("{:>4} ", count) } else { "".to_string() },
                    last_line);
            }
            break;
        }
        if beginning {
            last_line = line.clone();
        }
        if last_line.lines().collect::<String>() == line.lines().collect::<String>() {
            count += 1;
        } else {
            if config.out_file.as_ref() != None {
                to_be_written += &format!("{}{}",
                    if config.count { format!("{:>4} ", count) } else { "".to_string() },
                    last_line);
            } else {
            print!("{}{}",
                if config.count { format!("{:>4} ", count) } else { "".to_string() },
                last_line);
            }
            last_line = line.clone();
            count = 1;
        }
        line.clear();
        beginning = false;
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _   => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}