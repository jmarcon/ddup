//! Tree statistics builder.

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use crate::{
    DirHash, DirNode, DupFileGroup, DupGroup, DupStatus, FileHash, NodeKind, TreeStats,
    scanner::dir_dups::DirMeta,
};

pub(crate) fn build_tree_stats(
    nodes: &[DirNode],
    file_hashes: &HashMap<PathBuf, FileHash>,
    _file_sizes: &HashMap<PathBuf, u64>,
    dir_hashes: &HashMap<PathBuf, DirHash>,
    dir_metadata: &HashMap<PathBuf, DirMeta>,
    dir_groups: &[DupGroup],
    file_groups: &[DupFileGroup],
    roots: &[PathBuf],
) -> Vec<TreeStats> {
    let dir_group_by_path = dir_groups
        .iter()
        .flat_map(|group| {
            group
                .entries
                .iter()
                .map(|entry| (entry.path.clone(), group.id))
        })
        .collect::<HashMap<_, _>>();
    let file_group_by_path = file_groups
        .iter()
        .flat_map(|group| {
            group.entries.iter().filter_map(|entry| {
                entry
                    .suppressed_by_dir_group
                    .is_none()
                    .then_some((entry.path.clone(), group.id))
            })
        })
        .collect::<HashMap<_, _>>();

    let mut rows = Vec::new();
    for node in nodes {
        let dir_group_id = dir_group_by_path.get(&node.path).copied().flatten();
        let meta = dir_metadata.get(&node.path);
        let node_depth = depth(roots, &node.path);
        rows.push(TreeStats {
            path: node.path.clone(),
            kind: NodeKind::Dir,
            parent_path: parent_path(roots, &node.path),
            depth: node_depth,
            size_bytes: 0,
            size_recursive: meta.map_or(0, |value| value.size_bytes),
            file_count_recursive: meta.map_or(0, |value| value.file_count),
            extension: None,
            dup_status: if dir_group_id.is_some() {
                DupStatus::DupDir
            } else {
                DupStatus::Unique
            },
            dir_group_id,
            file_group_id: None,
            wasted_bytes: 0,
            content_hash: dir_hashes
                .get(&node.path)
                .map(|value| value.as_str().to_owned()),
        });

        for file in &node.files {
            let file_group_id = file_group_by_path.get(&file.path).copied().flatten();
            rows.push(TreeStats {
                path: file.path.clone(),
                kind: NodeKind::File,
                parent_path: Some(node.path.clone()),
                depth: depth(roots, &file.path),
                size_bytes: file.size,
                size_recursive: file.size,
                file_count_recursive: 1,
                extension: extension(&file.path),
                dup_status: if file_group_id.is_some() {
                    DupStatus::DupFile
                } else {
                    DupStatus::Unique
                },
                dir_group_id: None,
                file_group_id,
                wasted_bytes: 0,
                content_hash: file_hashes
                    .get(&file.path)
                    .map(|value| value.as_str().to_owned()),
            });
        }
    }

    let dup_dirs = dir_group_by_path.keys().cloned().collect::<HashSet<_>>();
    let dup_files = file_group_by_path.keys().cloned().collect::<HashSet<_>>();
    for row in &mut rows {
        row.wasted_bytes = wasted_bytes(row, dir_groups, file_groups);
        if row.kind == NodeKind::Dir && row.dup_status == DupStatus::Unique {
            let has_duplicate_descendant = dup_dirs
                .iter()
                .chain(dup_files.iter())
                .any(|path| path != &row.path && path.starts_with(&row.path));
            if has_duplicate_descendant {
                row.dup_status = DupStatus::Partial;
            }
        }
    }
    let partial_waste = rows
        .iter()
        .filter(|row| row.wasted_bytes > 0)
        .map(|row| (row.path.clone(), row.wasted_bytes))
        .collect::<Vec<_>>();
    for row in &mut rows {
        if row.dup_status == DupStatus::Partial {
            row.wasted_bytes = partial_waste
                .iter()
                .filter(|(path, _)| path.starts_with(&row.path) && path != &row.path)
                .map(|(_, waste)| *waste)
                .sum();
        }
    }
    rows.sort_by(|a, b| a.path.cmp(&b.path));
    rows
}

fn depth(roots: &[PathBuf], path: &Path) -> u32 {
    matching_root(path, roots)
        .and_then(|root| path.strip_prefix(root).ok())
        .map_or(0, |relative| {
            u32::try_from(relative.components().count()).unwrap_or(u32::MAX)
        })
}

fn parent_path(roots: &[PathBuf], path: &Path) -> Option<PathBuf> {
    (matching_root(path, roots) != Some(path))
        .then(|| path.parent().map(Path::to_path_buf))
        .flatten()
}

fn matching_root<'a>(path: &Path, roots: &'a [PathBuf]) -> Option<&'a Path> {
    roots
        .iter()
        .filter(|root| path.starts_with(root))
        .max_by_key(|root| root.components().count())
        .map(PathBuf::as_path)
}

fn extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|value| value.to_str())
        .map(str::to_lowercase)
}

fn wasted_bytes(row: &TreeStats, dir_groups: &[DupGroup], file_groups: &[DupFileGroup]) -> u64 {
    match row.dup_status {
        DupStatus::DupDir => dir_groups
            .iter()
            .find(|group| group.id == row.dir_group_id)
            .map_or(0, |group| {
                group.size_bytes * group.entries.len().saturating_sub(1) as u64
            }),
        DupStatus::DupFile => file_groups
            .iter()
            .find(|group| group.id == row.file_group_id)
            .map_or(0, |group| {
                group.size_bytes * group.entries.len().saturating_sub(1) as u64
            }),
        DupStatus::Unique | DupStatus::Partial => 0,
    }
}
