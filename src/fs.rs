use std::{
    collections::{BTreeMap, HashMap},
    ffi::OsString,
    fmt::Debug,
    fs::{self, read_to_string},
    io::Read,
    path::PathBuf,
    time::Duration,
};

use fuser::{FileAttr, Filesystem};
use libc::ENOENT;
use log::{debug, trace};

use crate::{Chive, from_metadata_to_fileattr};

pub struct ChiveFS {
    root: (PathBuf, FileAttr),
    entries: BTreeMap<OsString, (u64, FileAttr)>,
    chives: BTreeMap<OsString, (u64, FileAttr)>,
    // build during lookup, or is elsewhere better?
    // TODO don't clone again for this reverse lookup?
    ino_cache: HashMap<u64, OsString>,
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
        let (entries, chives) = fs::read_dir(&path)
            .expect("couldn't open {self.path} for listing")
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|e| e.is_file()))
            .enumerate()
            .map(|(ino, e)| {
                (
                    e.file_name(),
                    (
                        ino as u64 + 2,
                        from_metadata_to_fileattr(&e.metadata().unwrap()),
                    ),
                )
            })
            .partition(|(name, _)| !name.to_string_lossy().contains("chive"));

        debug!("entries: {:#?}", entries);
        debug!("chives: {:#?}", chives);

        let meta = from_metadata_to_fileattr(&path.metadata().unwrap());
        Self {
            root: (path, meta),
            entries,
            chives,
            ino_cache: HashMap::new(),
        }
    }

    fn update(&mut self) {
        todo!()
    }

    fn commit(&self) {
        todo!()
    }
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
            1 => reply.attr(&TTL, &self.root.1),
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
        // trace!("readdir");
        debug!(target: "readdir", "offset: {offset}");
        if ino != 1 {
            reply.error(ENOENT);
            return;
        }

        for (i, (name, (ino, file_attr))) in self
            .entries
            .iter()
            .chain(self.chives.iter())
            .enumerate()
            .skip(offset as usize)
        {
            if reply.add(*ino, (i + 1) as i64, file_attr.kind, name) {
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

        if parent != 1 {
            reply.error(ENOENT);
            return;
        }

        if let Some((name, (ino, file_attr))) = self.entries.get_key_value(name) {
            debug!("entry: ino: {ino}, attr: {file_attr:?}");
            let mut file_attr = *file_attr;
            file_attr.ino = *ino;
            reply.entry(
                &TTL, &file_attr,
                0, // TODO generation needs to be unique over NFS, we don't care rn
            );
            self.ino_cache.insert(*ino, name.clone());
        } else if let Some((name, (ino, file_attr))) = self.chives.get_key_value(name) {
            debug!("entry: ino: {ino}, attr: {file_attr:?}");
            let mut file_attr = *file_attr;
            file_attr.ino = *ino;
            reply.entry(
                &TTL, &file_attr,
                0, // TODO generation needs to be unique over NFS, we don't care rn
            );
            self.ino_cache.insert(*ino, name.clone());
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

        let (mut path, _) = self.root.clone();
        debug!("path root: {path:?}");
        path.push(self.ino_cache.get(&ino).unwrap());
        debug!("path file: {path:?}");
        let data: Vec<u8> = fs::read(path).unwrap().bytes().flatten().collect();

        reply.data(&data);
    }
}
