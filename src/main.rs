#![feature(path_add_extension, dir_entry_ext2)]

use std::{
    ffi::OsString,
    fs::{self, write},
    os::unix::fs::{DirEntryExt2, FileTypeExt, MetadataExt},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use clap::{Parser, Subcommand};
use fuser::{FileType, MountOption};
use libc::ENOENT;
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
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

    match &cli.command {
        Commands::Init { path, yes } => chive_init(path, *yes)?,
        Commands::Clear => chive_clear()?,
        Commands::Exec => chive_exec()?,
        Commands::Run { path, mount_point } => chive_run(path, mount_point)?,
        _ => unimplemented!(),
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Default)]
struct Chive {
    // hash
    tags: Vec<String>,
}

fn chive_init(path: &Path, yes: bool) -> anyhow::Result<()> {
    let entries = fs::read_dir(path)?;
    for entry in entries
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|e| e.is_file()))
        .filter(|e| !e.file_name().to_string_lossy().contains("chive"))
    {
        let prepend_dot = {
            let mut p = OsString::from(".");
            p.push(entry.file_name_ref());
            p
        };
        let sidecar_path = entry
            .path()
            .with_file_name(prepend_dot)
            .with_added_extension("chive");
        println!("{:?}", sidecar_path);
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

    Ok(())
}

fn chive_exec() -> anyhow::Result<()> {
    todo!();
    Ok(())
}

fn chive_run(path: &Path, mountpoint: &Path) -> anyhow::Result<()> {
    fuser::mount2(
        ChiveFS { path: path.into() },
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

// this is extremely gross
fn from_metadata_to_fileattr(value: std::fs::Metadata) -> fuser::FileAttr {
    fuser::FileAttr {
        ino: value.ino(),
        size: value.size(),
        blocks: value.blocks(),
        atime: value.accessed().unwrap_or(UNIX_EPOCH),
        mtime: value.modified().unwrap_or(UNIX_EPOCH),
        ctime: OffsetDateTime::from_unix_timestamp(value.ctime()).map_or(UNIX_EPOCH, |t| t.into()),
        crtime: value.created().unwrap_or(UNIX_EPOCH),
        kind: from_filetype_to_filetype(value.file_type()),
        perm: value.mode().try_into().unwrap(),
        nlink: value.nlink().try_into().unwrap(),
        uid: value.uid(),
        gid: value.gid(),
        rdev: value.rdev().try_into().unwrap(),
        blksize: value.blksize().try_into().unwrap(),
        // FIXME this is MacOS only, does rust even expose this?
        flags: 0,
    }
}

// this is even more gross
fn from_filetype_to_filetype(value: std::fs::FileType) -> fuser::FileType {
    use fuser::FileType;
    if value.is_dir() {
        FileType::Directory
    } else if value.is_file() {
        FileType::RegularFile
    } else if value.is_symlink() {
        FileType::Symlink
    } else if value.is_block_device() {
        FileType::BlockDevice
    } else if value.is_char_device() {
        FileType::CharDevice
    } else if value.is_fifo() {
        FileType::NamedPipe
    } else if value.is_socket() {
        FileType::Socket
    } else {
        unreachable!()
    }
}

struct ChiveFS {
    path: PathBuf,
}

impl fuser::Filesystem for ChiveFS {
    fn lookup(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) {
        trace!("lookup");
    }

    fn getattr(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: Option<u64>,
        reply: fuser::ReplyAttr,
    ) {
    }

    fn read(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        flags: i32,
        lock_owner: Option<u64>,
        reply: fuser::ReplyData,
    ) {
        trace!("read");
    }

    fn readdir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        mut reply: fuser::ReplyDirectory,
    ) {
        trace!("readdir");

        if ino != 1 {
            reply.error(ENOENT);
            return;
        }

        let mut entries = vec![
            (1, FileType::Directory, ".".to_string()),
            (1, FileType::Directory, "..".to_string()),
        ];

        let mut dir_entries: Vec<(u64, FileType, String)> = fs::read_dir(&self.path)
            .expect("couldn't read {self.path}")
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|e| e.is_file()))
            .filter(|e| e.file_name().to_string_lossy().contains("chive"))
            .enumerate()
            .map(|(i, e)| {
                (
                    (i + 2) as u64,
                    FileType::RegularFile,
                    e.file_name().into_string().unwrap(),
                )
            })
            .collect();

        entries.append(&mut dir_entries);

        debug!("{:?}", entries);

        // let mut ino = 2;
        for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
            if reply.add(entry.0, (i + 1) as i64, entry.1, entry.2) {
                // ino += 1;
                break;
            }
        }
    }
}
