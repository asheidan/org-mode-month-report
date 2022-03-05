use chrono::{self, Datelike};
use clap::Parser;
use regex::Regex;
use std::fs;
use std::io::{BufRead, self};

#[derive(Parser, Debug)]
#[clap(author, about, long_about = None)]
struct Options {
    /// The month to collect the report for
    #[clap(short, long)]
    date: Option<String>,

    /// The directory containing the worklog files
    #[clap(short, long, default_value_t = String::from("/home/asheidan/Worklog"))]
    worklog_dir: String,

    #[clap(long, default_value_t = String::from("%Y/%m %B"))]
    directory_pattern: String,
}

fn main() {
    let options = Options::parse();
    println!("{:?}", options);

    let date: chrono::NaiveDate = match options.date {
        Some(date_string) => {
            chrono::NaiveDate::parse_from_str(&date_string[..], "%Y-%m-%d").unwrap()
        }
        None => chrono::offset::Local::now().date().naive_local(),
    };
    //println!("{:?}", date);

    let month_directory_path = format!(
        "{}/{}",
        options.worklog_dir,
        date.format(&options.directory_pattern[..]).to_string(),
    );
    //println!("{}", month_directory_path);

    let interval_pattern = date
        .format(
            r"CLOCK: \[(%Y-%m-\d{2} ... \d{2}:\d{2})\]--\[(\d{4}-\d{2}-\d{2} ... \d{2}:\d{2})\]",
        )
        .to_string();
    let interval_regex = Regex::new(&interval_pattern[..]).unwrap();

    let available_files = fs::read_dir(options.worklog_dir).unwrap()
        .into_iter()
        .chain(fs::read_dir(month_directory_path).unwrap().into_iter())
        .filter(|entry| match entry {
            Ok(file) => {
                match file.metadata() {
                    Ok(metadata) => {
                        let file_name = file.file_name().into_string().unwrap();
                        metadata.is_file() && !file_name.starts_with(".") && file_name.ends_with(".org")
                    },
                    _ => false,
                }
            },
            _ => false,
        })
        .filter_map(|entry| match entry {
            Ok(entry) => {
                let file = fs::File::open(entry.path()).unwrap();
                let reader = io::BufReader::new(file);
                Some(reader.lines())
            },
            _ => None,
        })
        .flatten()
        .filter_map(|s| {
            let line = &s[..];
            match interval_regex.captures(line) {
            }
        });

    for entry in available_files {
        match entry {
            Ok(file) => {
                println!("{:?}", file.path());
            },
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }
    println!("{}", "--- End of files");

    let stdin = std::io::stdin();
    let timestamps = stdin
        .lock()
        .lines()
        .filter_map(|s| {
            let line = &s.unwrap()[..];
            match interval_regex.captures(line) {
                Some(captures) => {
                    let timestamps = [&captures[1], &captures[2]];
                    let datetimes =
                        timestamps.map(|ts| {
                            match chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %a %H:%M") {
                                Ok(datetime) => Some(datetime),
                                Err(err) => {
                                    println!("{:?}", err.to_string());
                                    None
                                }
                            }
                        });
                    match datetimes {
                        [Some(start), Some(end)] => {
                            Some((start.date(), (end - start).num_seconds()))
                        }
                        _ => None,
                    }
                }
                None => None,
            }
        })
        .fold([0; 31], |mut acc, (date, duration)| {
            acc[(date.day() - 1) as usize] += duration;
            acc
        });
    for (index, timestamp) in timestamps.iter().enumerate() {
        println!(
            "{:?}: {:?}",
            index + 1,
            (*timestamp as f64 / 900.0).round() / 4.0
        );
    }
}
