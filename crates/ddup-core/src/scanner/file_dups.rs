//! File duplicate detection.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{DupFileEntry, DupFileGroup, EntryStatus, FileHash};

pub(crate) fn detect_file_duplicates(
    file_hashes: &HashMap<PathBuf, FileHash>,
    file_sizes: &HashMap<PathBuf, u64>,
) -> Vec<DupFileGroup> {
    let mut by_hash = HashMap::<FileHash, Vec<&Path>>::new();
    for (path, hash) in file_hashes {
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
            let size = *file_sizes.get(*first)?;
            let entries = paths
                .into_iter()
                .map(|path| DupFileEntry {
                    id: None,
                    path: path.to_path_buf(),
                    status: EntryStatus::Pending,
                    suppressed_by_dir_group: None,
                })
                .collect::<Vec<_>>();
            Some(DupFileGroup {
                id: None,
                file_hash: hash,
                size_bytes: size,
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

pub(crate) fn apply_smart_suppression(
    file_groups: &mut [DupFileGroup],
    dir_groups: &[crate::DupGroup],
) {
    for file_group in file_groups {
        for entry in &mut file_group.entries {
            entry.suppressed_by_dir_group = dir_groups.iter().find_map(|group| {
                let id = group.id?;
                group
                    .entries
                    .iter()
                    .any(|dir_entry| entry.path.starts_with(&dir_entry.path))
                    .then_some(id)
            });
        }
    }
}
