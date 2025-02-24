use std::{
    collections::{BTreeMap, HashMap, HashSet},
    ffi::OsString,
    fs::{self, read_to_string},
    path::PathBuf,
};

use fuser::FileAttr;
use log::{debug, trace};

use crate::{Chive, from_metadata_to_fileattr};

mod fuse;

#[derive(Debug)]
pub struct ChiveFS {
    root: (PathBuf, FileAttr),
    entries: BTreeMap<OsString, Entry>,
    // chives: Vec<Chive>,
    chives: BTreeMap<OsString, (Entry, Chive)>,
    // build during lookup, or is elsewhere better?
    // TODO don't clone again for this reverse lookup?
    ino_cache: HashMap<u64, OsString>,
}

#[derive(Debug)]
struct Entry {
    ino: u64,
    parent: u64,
    file_attr: FileAttr,
}

// #[derive(Debug)]
// enum Entry {
//     File((u64, FileAttr)),
//     Dir((u64, FileAttr, BTreeMap<OsString, (u64, FileAttr)>)),
// }

impl ChiveFS {
    pub fn new(path: PathBuf) -> Self {
        // TODO why does example also have "." and ".."?
        debug!("path: {path:?}");
        let (mut entries, chives): (BTreeMap<_, _>, BTreeMap<_, _>) = fs::read_dir(&path)
            .expect("couldn't open {self.path} for listing")
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|e| e.is_file()))
            .enumerate()
            .map(|(i, e)| {
                (
                    e.file_name(),
                    (Entry {
                        ino: i as u64,
                        parent: 0,
                        file_attr: from_metadata_to_fileattr(&e.metadata().unwrap()),
                    }),
                )
            })
            .partition(|(name, _)| !name.to_string_lossy().contains("chive"));

        let chives: BTreeMap<_, _> = chives
            .into_iter()
            .map(|(name, entry)| {
                let chive: Chive = toml::from_str(
                    &read_to_string({
                        let mut path = path.clone();
                        path.push(&name);
                        path
                    })
                    .unwrap(),
                )
                .unwrap();
                (name, (entry, chive))
            })
            .collect();

        for (name, (_, chive)) in chives {
            for tag in chive.tags {
                entries.insert(
                    tag,
                    Entry {
                        ino: entries.len() + 1,
                        parent: 0,
                        file_attr: FileAttr {
                            ino: (),
                            size: (),
                            blocks: (),
                            atime: (),
                            mtime: (),
                            ctime: (),
                            crtime: (),
                            kind: (),
                            perm: (),
                            nlink: (),
                            uid: (),
                            gid: (),
                            rdev: (),
                            blksize: (),
                            flags: (),
                        },
                    },
                )
            }
            let entry = entries
                .get_mut(name.slice_encoded_bytes(0..name.len() - 6))
                .unwrap();
        }

        debug!("entries: {:#?}", entries);
        debug!("chives: {:#?}", chives);

        let meta = from_metadata_to_fileattr(&path.metadata().unwrap());
        todo!();
        // Self {
        //     root: (path, meta),
        //     entries,
        //     // chives,
        //     ino_cache: HashMap::new(),
        // }
    }

    fn update(&mut self) {
        todo!()
    }

    fn commit(&self) {
        todo!()
    }
}
