use std::fs::File;
use std::io::{self, BufRead};
use std::error::Error;
use std::path::Path;
use clap::{App, Arg};
use itertools::Itertools;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("efflux")
        .version("v1.0-beta")
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("The input file")
                .required(true),
        )
        .arg(
            Arg::with_name("host")
                .short("h")
                .long("host")
                .value_name("host")
                .help("The splunk hostname e.g: foo.splunk.com")
                .required(true),
        )
        .arg(
            Arg::with_name("token")
                .short("t")
                .long("token")
                .value_name("token")
                .help("The splunk token")
                .required(true),
        )
        .arg(
            Arg::with_name("source")
                .short("s")
                .long("source")
                .value_name("source")
                .help("The splunk source")
                .default_value("efflux"),
        )
        .arg(
            Arg::with_name("sourcetype")
                .short("y")
                .long("sourcetype")
                .value_name("sourcetype")
                .help("The splunk sourcetype")
                .default_value("generic_single_line"),
        )
        .get_matches();

    let client = reqwest::blocking::Client::new();

    let host = matches.value_of("host").unwrap();
    let source = matches.value_of("source").unwrap();
    let sourcetype = matches.value_of("sourcetype").unwrap();
    let url = format!(
        "https://{}/services/collector/raw?source={}&sourcetype={}",
        host, source, sourcetype
    );

    let token = matches.value_of("token").unwrap();
    let auth = format!("Splunk {}", token);

    let file = matches.value_of("file").unwrap();

    for (n, lines) in lines_from_file(file)?
        .batching(|it| batch_lines(it))
        .enumerate()
    {
        let size = lines.len();
        let resp = client
            .post(&url)
            .body(lines)
            .header("Authorization", &auth)
            .send();

        println!(
            "request:{:?} size:{:#?} status:{:#?}",
            n,
            size,
            resp.unwrap().status()
        );
    }

    Ok(())
}

fn lines_from_file<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn batch_lines<I>(it: &mut I) -> Option<String>
where
    I: Iterator<Item = std::result::Result<String, io::Error>>,
    I: std::fmt::Debug,
{
    let max = 1024 * 950;
    let mut lines = String::with_capacity(max);

    let mut size = 0;
    while size < max {
        match it.next() {
            None => {
                break;
            }
            Some(x) => {
                let s = x.unwrap();
                size += s.len();
                lines.push_str(s.as_str());
                lines.push('\n');
            }
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines)
    }
}
