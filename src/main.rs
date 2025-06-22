use byte_unit::{Byte, UnitType};
use chrono::{DateTime, Utc};
use clap::Parser;
use owo_colors::OwoColorize;
use serde::Serialize;
use std::{
    collections::VecDeque,
    fs,
    path::{Path, PathBuf},
};
use strum::Display;
use tabled::{
    Table, Tabled,
    settings::{
        Color, Style,
        object::{Columns, Rows},
    },
};

#[derive(Debug, Display, Serialize)]
enum EntryType {
    File,
    Dir,
}

#[derive(Debug, Tabled, Serialize)]
struct FileEntry {
    name: String,
    e_type: EntryType,
    size: String,
    modifiled: String,
    path: String,
}

#[derive(Debug, Parser)]
#[command(version, long_about = "Best Ls command ever")]
struct Cli {
    path: Option<PathBuf>,
    #[arg(short, long)]
    json: bool,
    #[arg(short, long)]
    clarity: bool,
    #[arg(short, long)]
    recursive: bool,
}

fn main() {
    let cli = Cli::parse();

    let mut paths: VecDeque<PathBuf> = VecDeque::new();

    let path = cli.path.unwrap_or(PathBuf::from("."));

    paths.push_back(path);

    while !paths.is_empty() {
        let current_path = paths.pop_front().unwrap();
        if let Ok(does_exist) = fs::exists(&current_path) {
            if does_exist {
                let get_files = get_files(&current_path);
                if cli.recursive {
                    for file in &get_files {
                        if let EntryType::Dir = file.e_type {
                            paths.push_back(PathBuf::from(file.path.to_string()))
                        }
                    }
                }
                println!("{}", current_path.to_str().unwrap());
                if cli.json {
                    println!(
                        "{}",
                        serde_json::to_string(&get_files)
                            .unwrap_or("can't parse to json".to_string())
                    );
                    println!();
                } else {
                    if cli.clarity {
                        print_table(get_files, true);
                    } else {
                        print_table(get_files, false);
                    }
                }
            } else {
                println!("{}", "Path does't exist".red());
            }
        } else {
            println!("{}", "error reading directory".red());
        }
    }
}

fn get_files(path: &Path) -> Vec<FileEntry> {
    let mut data = Vec::default();
    if let Ok(read_dir) = fs::read_dir(path) {
        for entry in read_dir {
            if let Ok(file) = entry {
                get_meta(file, &mut data);
            }
        }
    }
    data
}

fn get_meta(file: fs::DirEntry, data: &mut Vec<FileEntry>) {
    if let Ok(meta) = fs::metadata(&file.path()) {
        data.push(FileEntry {
            name: file
                .file_name()
                .into_string()
                .unwrap_or("unknown name".into()),
            e_type: if meta.is_dir() {
                EntryType::Dir
            } else {
                EntryType::File
            },
            size: meta.len().to_string(),
            modifiled: if let Ok(modi) = meta.modified() {
                let date: DateTime<Utc> = modi.into();
                format!("{}", date.format("%a %b %e %Y"))
            } else {
                String::default()
            },
            path: file.path().into_os_string().into_string().unwrap(),
        });
    }
}

fn print_table(mut get_files: Vec<FileEntry>, clear: bool) {
    for file_entry in &mut get_files {
        if clear {
            file_entry.size = Byte::from_u64(file_entry.size.parse::<u64>().unwrap())
                .get_appropriate_unit(UnitType::Binary)
                .to_string();
        } else {
            file_entry.size.push_str(" B");
        }
    }
    let mut table = Table::new(get_files);
    table.with(Style::rounded());
    table.modify(Columns::first(), Color::FG_BRIGHT_CYAN);
    table.modify(Columns::one(2), Color::FG_BRIGHT_MAGENTA);
    table.modify(Columns::one(3), Color::FG_BRIGHT_RED);
    table.modify(Rows::first(), Color::FG_BRIGHT_GREEN);
    println!("{}", table);
}
