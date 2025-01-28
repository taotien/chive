#![feature(path_add_extension, dir_entry_ext2)]

use std::{
    ffi::OsString,
    fs::{self},
    os::unix::fs::{DirEntryExt2, FileTypeExt, MetadataExt},
    time::UNIX_EPOCH,
};

use clap::{Parser, Subcommand};
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
        path: Option<String>,
        #[arg(short, long)]
        yes: bool,
    },
    Clear,
    Exec,
    Run,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { path, yes } => chive_init(&path.clone().unwrap_or(".".into()), *yes)?,
        Commands::Clear => chive_clear()?,
        Commands::Exec => chive_exec()?,
        Commands::Run => todo!(),
        _ => unimplemented!(),
    }

    Ok(())
}

fn chive_init(path: &str, yes: bool) -> anyhow::Result<()> {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            dbg!("{}", entry.metadata()?);
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

    Ok(())
}

fn chive_clear() -> anyhow::Result<()> {
    Ok(())
}

fn chive_exec() -> anyhow::Result<()> {
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
        // FIXME
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

struct ChiveFS;

impl fuser::Filesystem for ChiveFS {
    fn lookup(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) {
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
    }
    fn readdir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        reply: fuser::ReplyDirectory,
    ) {
    }
}
