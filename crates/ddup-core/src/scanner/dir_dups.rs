//! Directory duplicate detection.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{DirHash, DupEntry, DupGroup, EntryStatus};

#[derive(Clone, Debug)]
pub(crate) struct DirMeta {
    pub(crate) file_count: u64,
    pub(crate) size_bytes: u64,
}

pub(crate) fn detect_dir_duplicates(
    dir_hashes: &HashMap<PathBuf, DirHash>,
    dir_metadata: &HashMap<PathBuf, DirMeta>,
) -> Vec<DupGroup> {
    let mut by_hash = HashMap::<DirHash, Vec<&Path>>::new();
    for (path, hash) in dir_hashes {
        by_hash
            .entry(hash.clone())
            .or_default()
            .push(path.as_path());
    }

    let mut groups = by_hash
        .into_iter()
        .filter_map(|(hash, mut paths)| {
            if paths.len() < 2 {
                return None;
            }
            paths.sort();
            let first = paths.first()?;
            let meta = dir_metadata.get(*first)?;
            let entries = paths
                .into_iter()
                .map(|path| DupEntry {
                    id: None,
                    path: path.to_path_buf(),
                    status: EntryStatus::Pending,
                    dir_hash: hash.clone(),
                })
                .collect::<Vec<_>>();
            Some(DupGroup {
                id: None,
                dir_hash: hash,
                file_count: meta.file_count,
                size_bytes: meta.size_bytes,
                entries,
            })
        })
        .collect::<Vec<_>>();
    groups.sort_by(|a, b| a.entries[0].path.cmp(&b.entries[0].path));
    for (index, group) in groups.iter_mut().enumerate() {
        group.id = i64::try_from(index + 1).ok();
    }
    groups
}
