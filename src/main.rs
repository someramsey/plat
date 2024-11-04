#![allow(unused)]

mod loader;

use clap::{Arg, Command};
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Seek, Write};
use std::path::PathBuf;

fn get_data_file() -> File {
    let path = env::current_exe()
        .expect("Get executable path")
        .parent()
        .expect("Get executable directory")
        .join("templates");

    return OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(path)
        .expect("Open data file");
}

fn get_template_config_path() -> PathBuf {
    let path = env::current_dir()
        .expect("Get current directory")
        .join(".plat");

    return path;
}

fn init_template_config(path: &PathBuf) {
    if path.exists() {
        return;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)
        .expect("Open template config file");

    let current_dir = env::current_dir()
        .expect("Get current directory");
    let template_name = current_dir
        .file_name()
        .expect("Get current directory name")
        .to_str()
        .expect("Convert directory name to string");

    writeln!(file, "${}", template_name)
        .expect("Write template name to config file");
}


fn unlink() {
    let mut data_file = get_data_file();

    let config_path = get_template_config_path();
    let config_path_str = config_path.to_str()
        .expect("Convert template path to string");

    let reader = BufReader::new(&data_file);
    let lines: Vec<String> = reader.lines()
        .map(|line| line.expect("Read line from data file"))
        .filter(|line| line != config_path_str)
        .collect();

    data_file.set_len(0).expect("Truncate data file");

    data_file.seek(std::io::SeekFrom::Start(0))
        .expect("Seek to beginning of data file");

    for line in lines {
        writeln!(data_file, "{}", line)
            .expect("Write line to data file");
    }

    println!("Template unlinked successfully.");
}

fn link() {
    let mut data_file = get_data_file();

    let config_path = get_template_config_path();
    let config_path_str = config_path.to_str()
        .expect("Convert template path to string");

    init_template_config(&config_path);

    let reader = BufReader::new(&data_file);
    let contains = reader.lines()
        .map(|line| line.expect("Read line from data file"))
        .any(|line| line == config_path_str);

    if (!contains) {
        writeln!(data_file, "{}", config_path_str)
            .expect("Write template path to data file");

        println!("Template linked successfully.");
    } else {
        println!("Template already linked.");
    }
}

fn main() {
    let app = Command::new("plat")
        .version("1.0")
        .subcommand(
            Command::new("load")
                .alias("l")
                .about("Loads a template")
                .arg(Arg::new("name")
                    .required(true)
                    .help("The name of the template to load")
                    .index(1)),
        )
        .subcommand(Command::new("link")
            .about("Links the current directory as a template"))
        .subcommand(Command::new("unlink")
            .about("Unlinks the current directory as a template"));

    let matches = app.clone().get_matches();

    match matches.subcommand() {
        Some(("link", _)) => link(),
        Some(("unlink", _)) => unlink(),
        Some(("load", load_matches)) => {
            loader::load(load_matches.get_one::<String>("name"));
        }
        _ => {
            app.clone().print_help().expect("Print help");
            std::process::exit(0);
        }
    }
}