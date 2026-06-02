//! Scan orchestration.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use chrono::Utc;
use rayon::prelude::*;

use super::{
    dir_dups::{DirMeta, detect_dir_duplicates},
    file_dups::{apply_smart_suppression, detect_file_duplicates},
    tree_stats::build_tree_stats,
};
use crate::{
    DirHash, DirNode, DupFileGroup, DupGroup, FileEntry, FileHash, ProgressTx, Result, Scan,
    ScanEvent, ScanMode, TreeStats, WalkConfig, hash_dir, hash_file, progress::send_event,
    walk_with_errors,
};

/// Scan result.
#[derive(Clone, Debug)]
pub struct ScanResult {
    /// Duplicate directory groups.
    pub dir_groups: Vec<DupGroup>,
    /// Duplicate file groups.
    pub file_groups: Vec<DupFileGroup>,
    /// Tree statistics.
    pub tree_stats: Vec<TreeStats>,
    /// Scan summary.
    pub summary: Scan,
}

/// Scans a filesystem tree for duplicate directories and files.
pub fn scan(
    root: &Path,
    cfg: &WalkConfig,
    mode: ScanMode,
    progress: Option<ProgressTx>,
) -> Result<ScanResult> {
    let (nodes, walk_errors) = walk_with_errors(root, cfg)?;
    for message in walk_errors {
        send_event(
            progress.as_ref(),
            ScanEvent::Error {
                path: None,
                message,
            },
        );
    }
    send_event(
        progress.as_ref(),
        ScanEvent::Started {
            total_dirs_estimate: nodes.len(),
        },
    );

    let mut file_hashes = HashMap::<PathBuf, FileHash>::new();
    let mut file_sizes = HashMap::<PathBuf, u64>::new();
    let mut dir_hashes = HashMap::<PathBuf, DirHash>::new();
    let mut dir_metadata = HashMap::<PathBuf, DirMeta>::new();

    for (index, node) in nodes.iter().enumerate() {
        for file in &node.files {
            file_sizes.insert(file.path.clone(), file.size);
        }
        for (path, result) in hash_files_parallel(&node.files) {
            match result {
                Ok(hash) => {
                    file_hashes.insert(path, hash);
                }
                Err(error) => send_event(
                    progress.as_ref(),
                    ScanEvent::Error {
                        path: Some(path),
                        message: error.to_string(),
                    },
                ),
            }
        }

        let direct_file_hashes = node
            .files
            .iter()
            .filter_map(|file| file_hashes.get(&file.path).cloned())
            .collect::<Vec<_>>();
        let child_dir_hashes = node
            .child_dirs
            .iter()
            .filter_map(|path| dir_hashes.get(path).cloned())
            .collect::<Vec<_>>();
        let dir_hash = hash_dir(&direct_file_hashes, &child_dir_hashes);
        let (file_count, size_bytes) = dir_meta(node, &dir_metadata);
        dir_hashes.insert(node.path.clone(), dir_hash);
        dir_metadata.insert(
            node.path.clone(),
            DirMeta {
                file_count,
                size_bytes,
            },
        );
        send_event(
            progress.as_ref(),
            ScanEvent::DirHashed {
                path: node.path.clone(),
                current: index + 1,
            },
        );
    }

    let dir_groups = detect_dir_duplicates(&dir_hashes, &dir_metadata);
    let mut file_groups = detect_file_duplicates(&file_hashes, &file_sizes);
    if mode == ScanMode::Smart {
        apply_smart_suppression(&mut file_groups, &dir_groups);
    }
    send_event(
        progress.as_ref(),
        ScanEvent::FileDupsComputed {
            count: file_groups.len(),
        },
    );

    let tree_stats = build_tree_stats(
        &nodes,
        &file_hashes,
        &file_sizes,
        &dir_hashes,
        &dir_metadata,
        &dir_groups,
        &file_groups,
        root,
    );
    send_event(progress.as_ref(), ScanEvent::TreeStatsBuilt);

    let summary = Scan {
        id: None,
        root_path: root.to_path_buf(),
        scanned_at: Utc::now(),
        scan_mode: mode,
        total_dirs: nodes.len() as u64,
        total_files: file_sizes.len() as u64,
        wasted_bytes: wasted_bytes(&dir_groups, &file_groups, mode),
    };
    send_event(
        progress.as_ref(),
        ScanEvent::Finished {
            summary: summary.clone(),
        },
    );

    Ok(ScanResult {
        dir_groups,
        file_groups,
        tree_stats,
        summary,
    })
}

fn hash_files_parallel(files: &[FileEntry]) -> Vec<(PathBuf, Result<FileHash>)> {
    files
        .par_iter()
        .map(|file| (file.path.clone(), hash_file(&file.path)))
        .collect()
}

fn dir_meta(node: &DirNode, dir_metadata: &HashMap<PathBuf, DirMeta>) -> (u64, u64) {
    let direct_count = node.files.len() as u64;
    let direct_size = node.files.iter().map(|file| file.size).sum::<u64>();
    node.child_dirs
        .iter()
        .filter_map(|path| dir_metadata.get(path))
        .fold((direct_count, direct_size), |(count, size), meta| {
            (count + meta.file_count, size + meta.size_bytes)
        })
}

fn wasted_bytes(dir_groups: &[DupGroup], file_groups: &[DupFileGroup], mode: ScanMode) -> u64 {
    let dir_waste = dir_groups
        .iter()
        .map(|group| group.size_bytes * group.entries.len().saturating_sub(1) as u64)
        .sum::<u64>();
    let file_waste = file_groups
        .iter()
        .map(|group| {
            let count = if mode == ScanMode::Smart {
                group
                    .entries
                    .iter()
                    .filter(|entry| entry.suppressed_by_dir_group.is_none())
                    .count()
            } else {
                group.entries.len()
            };
            group.size_bytes * count.saturating_sub(1) as u64
        })
        .sum::<u64>();
    dir_waste + file_waste
}
