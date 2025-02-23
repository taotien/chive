use std::{
    collections::{BTreeMap, HashMap},
    ffi::OsString,
    fmt::Debug,
    fs::{self, File},
    io::Read,
    path::PathBuf,
    time::Duration,
};

use fuser::{FileAttr, Filesystem};
use libc::ENOENT;
use log::{debug, trace};

use crate::from_metadata_to_fileattr;

type Inode = u64;

pub struct ChiveFS {
    pub path: PathBuf,
    entries: BTreeMap<OsString, FileAttr>,
    // build during lookup, or is elsewhere better?
    // TODO don't clone again for this reverse lookup?
    ino_cache: HashMap<Inode, OsString>,
}

impl Debug for ChiveFS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.entries.keys())
    }
}

const TTL: Duration = Duration::from_secs(1);

impl ChiveFS {
    pub fn new(path: PathBuf) -> Self {
        // TODO why does example also have "." and ".."?
        debug!("path: {path:?}");
        let entries = BTreeMap::from_iter(
            fs::read_dir(&path)
                .expect("couldn't read {self.path}")
                .flatten()
                .filter(|e| e.file_type().is_ok_and(|e| e.is_file()))
                // .filter(|e| e.file_name().to_string_lossy().contains("chive"))
                .map(|e| {
                    (
                        e.file_name(),
                        from_metadata_to_fileattr(&e.metadata().unwrap()),
                    )
                }),
        );
        debug!("entries: {:#?}", entries);

        Self {
            path,
            entries,
            ino_cache: HashMap::new(),
        }
    }

    fn update(&mut self) {}

    fn commit(&self) {}
}

impl Filesystem for ChiveFS {
    fn getattr(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _fh: Option<u64>,
        reply: fuser::ReplyAttr,
    ) {
        trace!("getattr");
        debug!("ino: {ino}");
        match ino {
            1 => reply.attr(
                &TTL,
                &from_metadata_to_fileattr(&self.path.metadata().unwrap()),
            ),
            //     2 =>
            _ => {
                debug!("unknown ino");
                reply.error(ENOENT);
            }
        }
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
            if reply.add(entry.1.ino, (i + 1) as i64, entry.1.kind, entry.0) {
                break;
            }
        }
        reply.ok();
    }

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
            && let Some((name, attr)) = self.entries.get_key_value(name)
        {
            debug!("entry: {attr:?}");
            reply.entry(
                &TTL, attr, 0, // TODO generation needs to be unique over NFS, we don't care rn
            );
            self.ino_cache.insert(attr.ino, name.clone());
        } else {
            reply.error(ENOENT)
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
        debug!("ino: {ino}");
        debug!("fh: {fh}");
        debug!("offset: {offset}");
        debug!("size: {size}");
        debug!("flags: {flags}");
        debug!("lock_owner: {lock_owner:?}");
        debug!("reply: {reply:?}");

        let mut path = self.path.clone();
        debug!("path root: {path:?}");
        path.push(self.ino_cache.get(&ino).unwrap());
        debug!("path file: {path:?}");
        let data: Vec<u8> = fs::read(path).unwrap().bytes().flatten().collect();

        reply.data(&data);
    }
}
