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
    eprintln!("{:?}", options);

    let date: chrono::NaiveDate = match options.date {
        Some(date_string) => {
            chrono::NaiveDate::parse_from_str(&date_string[..], "%Y-%m-%d").unwrap()
        }
        None => chrono::offset::Local::now().date().naive_local(),
    };
    //eprintln!("{:?}", date);

    let month_directory_path = format!(
        "{}/{}",
        options.worklog_dir,
        date.format(&options.directory_pattern[..]).to_string(),
    );
    //eprintln!("{}", month_directory_path);

    let interval_pattern = date
        .format(
            r"CLOCK: \[(%Y-%m-\d{2} ... \d{2}:\d{2})\]--\[(\d{4}-\d{2}-\d{2} ... \d{2}:\d{2})\]",
        )
        .to_string();
    let interval_regex = Regex::new(&interval_pattern[..]).unwrap();

    let available_paths = fs::read_dir(options.worklog_dir).unwrap()
        .into_iter()
        .chain(fs::read_dir(month_directory_path).unwrap().into_iter())
        .filter_map(|entry| match entry {
            Ok(file) => {
                let file_name_maybe = file.file_name().into_string();
                match file_name_maybe {
                    Ok(normalized_file_name) => {
                        match file.metadata() {
                            Ok(metadata) => {
                                match metadata.is_file() && !normalized_file_name.starts_with(".") && normalized_file_name.ends_with(".org") {
                                    true => Some(file.path()),
                                    false => None,
                                }
                            },
                            _ => None,
                        }
                    },
                    _ => None,
                }
            },
            _ => None,
        });
    let timestamps = available_paths
        .map(|file_path| fs::File::open(file_path))
        .filter_map(|maybe_file| match maybe_file {
            Ok(file) => Some(io::BufReader::new(file).lines()),
            Err(_) => None,
        })
        .flatten()
        .filter_map(|io_lines| match io_lines {
            Ok(s) => {
                let line = &s[..];
                match interval_regex.captures(line) {
                    Some(captures) => {
                        let dates = [&captures[1], &captures[2]].map(|ts|{
                            match chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %a %H:%M") {
                                Ok(datetime) => Some(datetime),
                                Err(err) => {
                                    eprintln!("{:?}", err.to_string());
                                    None
                                }
                            }
                        });
                        Some(dates)
                    },
                    _ => None,
                }
            }
            _ => None,
        })
        .filter_map(|datetimes| {
            match datetimes {
                [Some(start), Some(end)] => {
                    Some((start.date(), (end - start).num_seconds()))
                }
                _ => None,
            }
        });

    let mapped_timestamps = timestamps
        .fold([0; 31], |mut acc, (date, duration)| {
            acc[(date.day() - 1) as usize] += duration;
            acc
        })
        .map(|number| (number as f64 / 900.0).round() / 4.0);

    let mapped_strings = mapped_timestamps
        .map(|number| match number {
            f if (-1.0 ..= 0.0).contains(&f) => String::from(""),
            f => format!("{:.2}", f),
        });
    
    for (index, duration_string) in mapped_strings.iter().enumerate() {
        eprintln!(
            "{:2}:{:>6}",
            index + 1,
            duration_string,
        );
    }
    eprintln!("T:{:>7.2}", mapped_timestamps.iter().sum::<f64>());

    eprintln!("--------------------------------------------------------------------------------");
    let row = mapped_strings.join("\t");
    println!("{}", row);
}
