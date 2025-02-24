use std::{
    fs::{self},
    io::Read,
    time::Duration,
};

use fuser::Filesystem;
use libc::ENOENT;
use log::{debug, trace};

use super::ChiveFS;

const TTL: Duration = Duration::from_secs(1);

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
            1 => {
                let mut attr = self.root.1;
                attr.ino = 1;
                // debug!("reply attr: {:?}", attr);
                reply.attr(&TTL, &attr);
            }
            // 2 =>
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
            debug!("reply add: ino {ino}, name {name:?}, attr {file_attr:?}, offset: {i}");
            if reply.add(*ino, (i + 1) as i64, file_attr.kind, name) {
                break;
            }
        }
        if offset == 2 {
            reply.add(21, offset + 1, fuser::FileType::Directory, "dir");
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
