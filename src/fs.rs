use std::{
    collections::BTreeMap,
    ffi::OsString,
    fmt::Debug,
    fs::{self},
    path::PathBuf,
    time::{Duration, UNIX_EPOCH},
};

use fuser::{FileAttr, FileType, Filesystem};
use libc::ENOENT;
use log::{debug, trace};

use crate::from_metadata_to_fileattr;

pub struct ChiveFS {
    pub path: PathBuf,
    entries: BTreeMap<OsString, (u64, FileAttr)>,
}

impl Debug for ChiveFS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.entries.keys())
    }
}

const TTL: Duration = Duration::from_secs(1);

impl ChiveFS {
    pub fn new(path: PathBuf) -> Self {
        // let mut entries = HashMap::from([ ( OsString::from("."), ( 1, FileAttr { ino: todo!(), size: todo!(), blocks: todo!(), atime: todo!(), mtime: todo!(), ctime: todo!(), crtime: todo!(), kind: todo!(), perm: todo!(), nlink: todo!(), uid: todo!(), gid: todo!(), rdev: todo!(), blksize: todo!(), flags: todo!(), }, ), ), (OsString::from(".."), (1,)), ]);
        let mut entries = BTreeMap::new();

        debug!("{path:?}");
        entries.extend(
            fs::read_dir(&path)
                .expect("couldn't read {self.path}")
                .flatten()
                .filter(|e| e.file_type().is_ok_and(|e| e.is_file()))
                .filter(|e| e.file_name().to_string_lossy().contains("chive"))
                .enumerate()
                .map(|(i, e)| {
                    (
                        e.file_name(),
                        (
                            i as u64 + 2,
                            from_metadata_to_fileattr(&e.metadata().unwrap()),
                        ),
                    )
                }),
        );

        debug!("entries: {:#?}", entries.keys());

        Self { path, entries }
    }
}

impl Filesystem for ChiveFS {
    fn lookup(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) {
        trace!("lookup");
        debug!("parent: {parent}, name: {name:?}");

        if parent == 1
            && let Some(entry) = self.entries.get(name)
        {
            reply.entry(
                &TTL, &entry.1, 0, // TODO needs to be unique over NFS, we don't care rn
            )
        } else {
            reply.error(ENOENT)
        }
    }

    fn getattr(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _fh: Option<u64>,
        reply: fuser::ReplyAttr,
    ) {
        trace!("getattr");
        match ino {
            1 => reply.attr(
                &TTL,
                &from_metadata_to_fileattr(&self.path.metadata().unwrap()),
            ),
            //     2 =>
            _ => {
                debug!("unknown ino: {ino}");
                reply.error(ENOENT);
            }
        }
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
        _fh: u64,
        offset: i64,
        mut reply: fuser::ReplyDirectory,
    ) {
        trace!("readdir");
        debug!("offset: {offset}");
        if ino != 1 {
            reply.error(ENOENT);
            return;
        }

        for (i, entry) in self.entries.iter().enumerate().skip(offset as usize) {
            // debug!("iter'd: {entry:?}");
            if reply.add(
                entry.1.0,
                (i + 1) as i64,
                entry.1.1.kind,
                entry.0.slice_encoded_bytes(1..),
            ) {
                // debug!("replied: {entry:?}");
                break;
            }
        }
        reply.ok();
    }
}
