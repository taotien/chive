#![feature(let_chains)]

use std::{
    os::unix::fs::{FileTypeExt, MetadataExt},
    time::UNIX_EPOCH,
};

use time::OffsetDateTime;

pub mod fs;

// this is extremely gross
pub fn from_metadata_to_fileattr(value: &std::fs::Metadata) -> fuser::FileAttr {
    fuser::FileAttr {
        ino: value.ino(),
        size: value.size(),
        blocks: value.blocks(),
        atime: value.accessed().unwrap_or(UNIX_EPOCH),
        mtime: value.modified().unwrap_or(UNIX_EPOCH),
        ctime: OffsetDateTime::from_unix_timestamp(value.ctime()).map_or(UNIX_EPOCH, |t| t.into()),
        crtime: value.created().unwrap_or(UNIX_EPOCH),
        kind: from_filetype_to_filetype(&value.file_type()),
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
fn from_filetype_to_filetype(value: &std::fs::FileType) -> fuser::FileType {
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
