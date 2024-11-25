#![allow(unused)]
#![allow(clippy::needless_return)]

mod task;

use crate::task::evaluate::evaluate;
use task::tokenizer::tokenize::tokenize;
use clap::{Arg, Command};
use glob::glob;
use indicatif::ProgressBar;
use std::collections::HashMap;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, Write};
use std::path::PathBuf;
use std::sync::Arc;

fn open_data_file() -> File {
    let path = env::current_exe()
        .expect("Get current executable path")
        .parent()
        .expect("Get parent directory of executable")
        .join(".platdata");

    return OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path)
        .expect("Open config file");
}

fn read_data_file() -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();

    let mut data_file = open_data_file();
    let reader = BufReader::new(&data_file);

    for line in reader.lines() {
        let line = line.expect("Read line from file");

        if let Some(pos) = line.find('|') {
            let name = &line[..pos];
            let path = &line[pos + 1..];

            map.insert(name.to_string(), path.to_string());
        }
    }

    return map;
}

fn write_data_file(data: &HashMap<String, String>) {
    let mut data_file = open_data_file();

    data_file.set_len(0).expect("Truncate file");
    data_file
        .seek(std::io::SeekFrom::Start(0))
        .expect("Seek to start of file");

    for (name, path) in data {
        writeln!(data_file, "{}|{}", name, path).expect("Write to file");
    }
}

fn load(origin: PathBuf, target: PathBuf, progress_bar: ProgressBar) {
    let task_file_path = origin.join("task.plat");

    if task_file_path.exists() {
        let content = std::fs::read_to_string(&task_file_path).expect("Read task file");

        return;
    }

    let mut options = fs_extra::dir::CopyOptions::new()
        .copy_inside(true)
        .overwrite(true);

    fs_extra::dir::copy(&origin, &target, &options).expect("Copy files");
}

fn main() {
    let file = File::open(".platenv").unwrap();
    let mut reader = BufReader::new(&file);

    let mut data = String::new();
    reader.read_to_string(&mut data).unwrap();

    let tokens = tokenize(&data);
    // let instructions = parse_instructions(tokens);


    return;

    let app = Command::new("plat")
        .version("1.0")
        .subcommand(
            Command::new("load")
                .alias("l")
                .about("Loads a template")
                .arg(
                    Arg::new("name")
                        .required(true)
                        .help("The name of the template to load")
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("link")
                .arg(Arg::new("name").help("The name of the template").index(1))
                .about("Links the current directory as a template"),
        )
        .subcommand(Command::new("unlink").about("Unlinks the current directory as a template"))
        .subcommand(
            Command::new("list")
                .alias("ls")
                .about("Lists all linked templates"),
        );

    let matches = app.clone().get_matches();

    let current_dir = env::current_dir().expect("Get current directory");

    match matches.subcommand() {
        Some(("link", submatches)) => {
            let mut data = read_data_file();

            let name = submatches.get_one::<String>("name").map_or_else(
                || {
                    return dialoguer::Input::new()
                        .with_prompt("Enter a name for the template")
                        .interact()
                        .expect("Prompt for template name");
                },
                |name| name.trim().to_string(),
            );

            let path = current_dir
                .to_str()
                .expect("Convert current directory to string");

            if data.contains_key(&name) {
                println!(
                    "A template with the name '{}' already exists, please choose a different name",
                    name
                );
                return;
            }

            if data.values().any(|v| v == path) {
                println!("The template at '{}' is already linked.", path);
                return;
            }

            data.insert(name.to_string(), path.to_string());

            write_data_file(&data);

            println!("Template {} is now linked.", name);
        }

        Some(("unlink", _)) => {
            let mut data = read_data_file();

            let path = current_dir
                .to_str()
                .expect("Convert current directory to string");

            data.retain(|_, v| v != path);

            write_data_file(&data);

            println!("Template {} is now unlinked.", path);
        }

        Some(("load", load_matches)) => {
            let name = load_matches
                .get_one::<String>("name")
                .expect("Get name argument");
            let data = read_data_file();

            if let Some(path) = data.get(name) {
                let confirmed = dialoguer::Confirm::new()
                    .with_prompt(format!(
                        "Do you want to load the template '{}' into the current directory?",
                        name
                    ))
                    .interact()
                    .expect("Prompt confirm message before loading");

                if !confirmed {
                    return;
                }

                println!("Loading template from {}", path);

                let origin_path = PathBuf::from(path);
                let progress_bar = ProgressBar::new(100);

                load(origin_path, current_dir, progress_bar);

                println!("Finished loading template");
            } else {
                println!("Template '{}' was not found, try checking the linked templates with 'plat list'", name);
            }
        }

        Some(("list", _)) => {
            let data = read_data_file();

            if data.is_empty() {
                println!("No templates linked.");
            } else {
                println!("Linked templates:");

                for (name, path) in data {
                    println!("{} -> {}", name, path);
                }
            }
        }

        _ => {
            app.clone().print_help().expect("Print help");
            std::process::exit(0);
        }
    }
}
