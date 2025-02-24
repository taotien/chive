use std::{
    collections::{BTreeMap, HashMap},
    ffi::OsString,
    fs::{self},
    path::PathBuf,
};

use fuser::FileAttr;
use log::{debug, trace};

use crate::{Chive, from_metadata_to_fileattr};

mod fuse;

pub struct ChiveFS {
    root: (PathBuf, FileAttr),
    entries: BTreeMap<OsString, (u64, FileAttr)>,
    chives: BTreeMap<OsString, (u64, FileAttr)>,
    // build during lookup, or is elsewhere better?
    // TODO don't clone again for this reverse lookup?
    ino_cache: HashMap<u64, OsString>,
}

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
