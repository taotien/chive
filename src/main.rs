#![feature(path_add_extension, dir_entry_ext2)]

use std::{
    fs::{self, write},
    os::unix::fs::DirEntryExt2,
    path::{Path, PathBuf},
};

use chive::{Chive, fs::ChiveFS};
use clap::{Parser, Subcommand};
use fuser::MountOption;
use log::{debug, trace};

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Init {
        #[arg(short, long)]
        path: PathBuf,
        #[arg(short, long)]
        yes: bool,
    },
    Clear,
    Exec,
    Run {
        #[arg(short, long)]
        path: PathBuf,
        mount_point: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    debug!("cli: {cli:?}");
    match &cli.command {
        Commands::Init { path, yes } => chive_init(path, *yes)?,
        Commands::Clear => chive_clear()?,
        Commands::Exec => chive_exec()?,
        Commands::Run { path, mount_point } => chive_run(path, mount_point)?,
        _ => unimplemented!(),
    }

    Ok(())
}

fn chive_init(path: &Path, yes: bool) -> anyhow::Result<()> {
    trace!("init");
    let entries = fs::read_dir(path)?;
    for entry in entries
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|e| e.is_file()))
        .filter(|e| !e.file_name().to_string_lossy().contains("chive"))
    {
        debug!("entry: {entry:?}");
        let sidecar_path = entry
            .path()
            .with_file_name(entry.file_name_ref())
            .with_added_extension("chive");
        debug!("sidecar_path {sidecar_path:?}");
        if yes {
            let sidecar = Chive::default();
            write(sidecar_path, toml::to_string(&sidecar)?)?;
        } else {
            todo!()
        }
    }

    Ok(())
}

fn chive_clear() -> anyhow::Result<()> {
    todo!();

    // Ok(())
}

fn chive_exec() -> anyhow::Result<()> {
    todo!();
    // Ok(())
}

fn chive_run(path: &Path, mountpoint: &Path) -> anyhow::Result<()> {
    let chive = ChiveFS::new(path.into());
    fuser::mount2(
        chive,
        mountpoint,
        &[
            MountOption::RO,
            MountOption::FSName("ChiveFS".to_string()),
            // MountOption::AutoUnmount,
            // MountOption::AllowRoot,
        ],
    )?;

    Ok(())
}
