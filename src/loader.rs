use crate::get_data_file;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

fn load_template(path: &PathBuf) {
    println!("dw");
}

fn read_template_config_name(path: &PathBuf) -> Option<String> {
    let reader = BufReader::new(File::open(path)
        .expect("Open template config file"));

    return reader.lines()
        .map(|line| line.expect("Read line from config file"))
        .find(|line| line.starts_with("$"))
        .map(|line| line[1..].to_string());
}


pub fn load(name_res: Option<&String>) {
    let data_file = get_data_file();
    let reader = BufReader::new(&data_file);

    let mut template_paths = reader.lines()
        .map(|line| line.expect("Read line from data file"))
        .map(|path| PathBuf::from(path))
        .collect();

    if let Some(name) = name_res {
        let template_path = template_paths
            .find(|path| {
                if !path.exists() { return false; }

                let template_name = read_template_config_name(path)
                    .expect("Read template name from config file");

                return template_name == *name;
            });

        if let Some(path) = template_path {
            println!("Loading template: {}", name);
            load_template(&path);
            return;
        }

        println!("Template not found.");
    } else {
        let template_names: Vec<String> = template_paths
            .map(|path| read_template_config_name(&path)
                .expect("Read template name from config file"))
            .collect();

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose an option")
            .items(&template_names)
            .interact()
            .expect("Failed to select an option");

        let path = template_paths[selection];
        let name = &template_names[selection];

        println!("Loading template: {}", name);
    }
}