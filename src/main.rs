#![feature(path_add_extension, dir_entry_ext2)]

use std::{ffi::OsString, fs, os::unix::fs::DirEntryExt2};

use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(short, long)]
        path: Option<String>,
        #[arg(short, long)]
        yes: bool,
    },
    Clear,
    Exec,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init { path, yes }) => chive_init(&path.clone().unwrap_or(".".into()), *yes),
        Some(Commands::Clear) => chive_clear(),
        Some(Commands::Exec) => chive_exec(),
        _ => unimplemented!(),
    }
}

fn chive_init(path: &str, yes: bool) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                // println!("{:?}: {:?}", entry.path(), file_type);
                if file_type.is_file() {
                    let prepend_dot = {
                        let mut p = OsString::from(".");
                        p.push(entry.file_name_ref());
                        p
                    };
                    let sidecar = entry
                        .path()
                        .with_file_name(prepend_dot)
                        .with_added_extension("chive");
                    println!("{:?}", sidecar);
                }
            }
        }
    }
}

fn chive_clear() {}

fn chive_exec() {}
