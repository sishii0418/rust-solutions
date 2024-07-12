use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: usize,
    bytes: Option<usize>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("headr")
        .version("0.1.0")
        .author("Shunsuke Ishii")
        .about("Rust head")
        .arg(
            Arg::with_name("files")
                .value_name("FILES")
                .help("Input file(s)")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("lines")
                .value_name("LINES")
                .short("n")
                .long("lines")
                .help("Number of lines")
                .multiple(false)
                .takes_value(true)
                .default_value("10"),
        )
        .arg(
            Arg::with_name("bytes")
                .value_name("BYTES")
                .short("c")
                .long("bytes")
                .help("Number of bytes")
                .takes_value(true)
                .conflicts_with("lines"),
        )
        .get_matches();

    let lines = matches
        .value_of("lines")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal line count -- {}", e))?;

    let bytes = matches
        .value_of("bytes")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal byte count -- {}", e))?;

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        lines: lines.unwrap(),
        bytes,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let multiple = config.files.len() > 1;
    for filename in &config.files {
        if multiple {
            println!("==> {} <==", filename);
        }
        match open(&filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(mut file) => {
                match config.bytes {
                    Some(n) => {
                        let mut count = 0;
                        let mut buffer = vec![0u8; 1];
                        let mut line = vec![];
                        loop {
                            match file.read(&mut buffer) {
                                Ok(0) => {
                                    // EOF reached, break the loop
                                    print!("{}", String::from_utf8_lossy(&line));
                                    break;
                                },
                                Ok(_) => {
                                    // Successfully read a byte, process it
                                    line.push(buffer[0]);
                                    count += 1;
                                    if count >= n {
                                        print!("{}", String::from_utf8_lossy(&line));
                                        break;
                                    }
                                },
                                Err(e) => return Err(From::from(e)), // Handle read error
                            }
                        }
                    },
                    None    => {
                        let mut count = 0;
                        let mut buffer = vec![0u8; 1];
                        let mut line = vec![];
                        loop {
                            match file.read(&mut buffer) {
                                Ok(0) => {
                                    break;
                                },
                                Ok(_) => {
                                    // Successfully read a byte, process it
                                    line.push(buffer[0]);
                                    if buffer[0] == b'\n' {
                                        print!("{}", String::from_utf8(line.clone()).unwrap());
                                        line.clear();
                                        count += 1;
                                    }
                                    if count >= config.lines {
                                        break;
                                    }
                                },
                                Err(e) => return Err(From::from(e)), // Handle read error
                            }
                        }
                    }
                }
            }
        }
        if multiple && filename != config.files.last().unwrap() {
            println!();
        }
    }
    Ok(())
}

fn parse_positive_int(val: &str) -> MyResult<usize> {
    match val.parse() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(From::from(val)),
    }
}

#[test]
fn test_parse_positive_int() {
    // Ok as 3 is an integer
    let res = parse_positive_int("3");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);

    // Non-integer string causes an error
    let res = parse_positive_int("foo");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "foo".to_string());

    // 0 also causes an error
    let res = parse_positive_int("0");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "0".to_string());
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}