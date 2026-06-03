//! TUI tree model.

use std::path::{Path, PathBuf};

use ddup_core::{DupFileGroup, DupGroup, SortBy, SortConfig, SortOrder};

use crate::app::ViewMode;

/// TUI node kind.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NodeKind {
    /// Group root.
    GroupRoot,
    /// Directory entry.
    DirEntry,
    /// File entry.
    FileEntry,
    /// File leaf.
    FileLeaf,
}

/// TUI tree node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeNode {
    /// Path.
    pub path: PathBuf,
    /// Entry id.
    pub entry_id: Option<i64>,
    /// Display label.
    pub label: String,
    /// Visible depth.
    pub depth: usize,
    /// Kind.
    pub kind: NodeKind,
    /// Expanded state.
    pub expanded: bool,
    /// Children.
    pub children: Vec<TreeNode>,
    /// Backing group id.
    pub group_id: Option<i64>,
    /// Duplicate marker.
    pub dup_marker: String,
    /// Group entry count.
    pub entry_count: usize,
    /// Duplicate group size.
    pub size_bytes: u64,
    /// Recursive file count.
    pub file_count: Option<u64>,
    /// File or directory hash.
    pub content_hash: Option<String>,
}

/// Tree model.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TreeModel {
    /// Root nodes.
    pub roots: Vec<TreeNode>,
    /// Cursor index in flattened nodes.
    pub cursor: usize,
}

impl TreeModel {
    /// Flattens visible nodes.
    #[must_use]
    pub fn flatten_visible(&self) -> Vec<&TreeNode> {
        let mut out = Vec::new();
        for node in &self.roots {
            flatten(node, &mut out);
        }
        out
    }

    /// Moves cursor by signed delta.
    pub fn move_cursor(&mut self, delta: isize) {
        let len = self.flatten_visible().len();
        if len == 0 {
            self.cursor = 0;
            return;
        }
        self.cursor = self.cursor.saturating_add_signed(delta).min(len - 1);
    }

    /// Toggles selected node expansion.
    pub fn toggle_expand(&mut self) {
        let mut index = 0;
        toggle_at(&mut self.roots, self.cursor, &mut index);
    }

    /// Opens selected node.
    pub fn open_selected(&mut self) {
        let mut index = 0;
        set_expanded_at(&mut self.roots, self.cursor, &mut index, true);
    }

    /// Closes selected node.
    pub fn close_selected(&mut self) {
        let mut index = 0;
        set_expanded_at(&mut self.roots, self.cursor, &mut index, false);
    }

    /// Selects visible node by index.
    pub fn select(&mut self, index: usize) {
        let len = self.flatten_visible().len();
        if len > 0 {
            self.cursor = index.min(len - 1);
        }
    }

    /// Returns selected visible node.
    #[must_use]
    pub fn selected(&self) -> Option<&TreeNode> {
        self.flatten_visible().get(self.cursor).copied()
    }
}

/// Builds tree model for a view.
#[must_use]
pub fn build_tree(
    view: ViewMode,
    dir_groups: &[DupGroup],
    file_groups: &[DupFileGroup],
    sort: SortConfig,
    root: &Path,
) -> TreeModel {
    let roots = match view {
        ViewMode::DirsDuplicated => sorted_dir_groups(dir_groups, sort)
            .into_iter()
            .map(|group| TreeNode {
                path: PathBuf::new(),
                entry_id: None,
                label: format!("Dir group {}", group.id.unwrap_or_default()),
                depth: 0,
                kind: NodeKind::GroupRoot,
                expanded: true,
                group_id: group.id,
                dup_marker: "dir".to_owned(),
                children: group
                    .entries
                    .iter()
                    .map(|entry| TreeNode {
                        path: entry.path.clone(),
                        entry_id: entry.id,
                        label: relative_label(&entry.path, root),
                        depth: 1,
                        kind: NodeKind::DirEntry,
                        expanded: false,
                        children: Vec::new(),
                        group_id: group.id,
                        dup_marker: entry.status.to_string(),
                        entry_count: group.entries.len(),
                        size_bytes: group.size_bytes,
                        file_count: Some(group.file_count),
                        content_hash: Some(entry.dir_hash.as_str().to_owned()),
                    })
                    .collect(),
                entry_count: group.entries.len(),
                size_bytes: group.size_bytes,
                file_count: Some(group.file_count),
                content_hash: Some(group.dir_hash.as_str().to_owned()),
            })
            .collect(),
        ViewMode::FilesDuplicatedSmart | ViewMode::FilesDuplicatedFlat => {
            sorted_file_groups(file_groups, sort)
                .into_iter()
                .map(|group| TreeNode {
                    path: PathBuf::new(),
                    entry_id: None,
                    label: format!("File group {}", group.id.unwrap_or_default()),
                    depth: 0,
                    kind: NodeKind::GroupRoot,
                    expanded: true,
                    group_id: group.id,
                    dup_marker: "file".to_owned(),
                    children: group
                        .entries
                        .iter()
                        .map(|entry| TreeNode {
                            path: entry.path.clone(),
                            entry_id: entry.id,
                            label: relative_label(&entry.path, root),
                            depth: 1,
                            kind: NodeKind::FileEntry,
                            expanded: false,
                            children: Vec::new(),
                            group_id: group.id,
                            dup_marker: entry.status.to_string(),
                            entry_count: group.entries.len(),
                            size_bytes: group.size_bytes,
                            file_count: None,
                            content_hash: Some(group.file_hash.as_str().to_owned()),
                        })
                        .collect(),
                    entry_count: group.entries.len(),
                    size_bytes: group.size_bytes,
                    file_count: None,
                    content_hash: Some(group.file_hash.as_str().to_owned()),
                })
                .collect()
        }
    };
    TreeModel { roots, cursor: 0 }
}

fn sorted_dir_groups(groups: &[DupGroup], sort: SortConfig) -> Vec<&DupGroup> {
    let mut groups = groups.iter().collect::<Vec<_>>();
    groups.sort_by(|a, b| match sort.by {
        SortBy::Name => a
            .entries
            .first()
            .map(|entry| &entry.path)
            .cmp(&b.entries.first().map(|entry| &entry.path)),
        SortBy::TotalSize => dir_group_consumed_bytes(a).cmp(&dir_group_consumed_bytes(b)),
        SortBy::FileCount => a.entries.len().cmp(&b.entries.len()),
    });
    if sort.order == SortOrder::Desc {
        groups.reverse();
    }
    groups
}

fn sorted_file_groups(groups: &[DupFileGroup], sort: SortConfig) -> Vec<&DupFileGroup> {
    let mut groups = groups.iter().collect::<Vec<_>>();
    groups.sort_by(|a, b| match sort.by {
        SortBy::Name => a
            .entries
            .first()
            .map(|entry| &entry.path)
            .cmp(&b.entries.first().map(|entry| &entry.path)),
        SortBy::TotalSize => file_group_consumed_bytes(a).cmp(&file_group_consumed_bytes(b)),
        SortBy::FileCount => a.entries.len().cmp(&b.entries.len()),
    });
    if sort.order == SortOrder::Desc {
        groups.reverse();
    }
    groups
}

fn dir_group_consumed_bytes(group: &DupGroup) -> u64 {
    group.size_bytes.saturating_mul(group.entries.len() as u64)
}

fn file_group_consumed_bytes(group: &DupFileGroup) -> u64 {
    group.size_bytes.saturating_mul(group.entries.len() as u64)
}

fn relative_label(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .map_or(path, |relative| relative)
        .display()
        .to_string()
}

fn flatten<'a>(node: &'a TreeNode, out: &mut Vec<&'a TreeNode>) {
    out.push(node);
    if node.expanded {
        for child in &node.children {
            flatten(child, out);
        }
    }
}

fn toggle_at(nodes: &mut [TreeNode], target: usize, index: &mut usize) -> bool {
    for node in nodes {
        if *index == target {
            node.expanded = !node.expanded;
            return true;
        }
        *index += 1;
        if node.expanded && toggle_at(&mut node.children, target, index) {
            return true;
        }
    }
    false
}

fn set_expanded_at(
    nodes: &mut [TreeNode],
    target: usize,
    index: &mut usize,
    expanded: bool,
) -> bool {
    for node in nodes {
        if *index == target {
            node.expanded = expanded;
            return true;
        }
        *index += 1;
        if node.expanded && set_expanded_at(&mut node.children, target, index, expanded) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

    use ddup_core::{DirHash, DupEntry, EntryStatus};

    use super::*;

    fn model() -> TreeModel {
        TreeModel {
            cursor: 0,
            roots: vec![TreeNode {
                path: PathBuf::from("root"),
                entry_id: None,
                label: "root".to_owned(),
                depth: 0,
                kind: NodeKind::GroupRoot,
                expanded: true,
                children: vec![TreeNode {
                    path: PathBuf::from("child"),
                    entry_id: Some(1),
                    label: "child".to_owned(),
                    depth: 1,
                    kind: NodeKind::DirEntry,
                    expanded: false,
                    children: Vec::new(),
                    group_id: Some(1),
                    dup_marker: "pending".to_owned(),
                    entry_count: 2,
                    size_bytes: 100,
                    file_count: Some(3),
                    content_hash: Some("child-hash".to_owned()),
                }],
                group_id: Some(1),
                dup_marker: "dir".to_owned(),
                entry_count: 2,
                size_bytes: 100,
                file_count: Some(3),
                content_hash: Some("root-hash".to_owned()),
            }],
        }
    }

    #[test]
    fn flatten_visible_includes_expanded_children() {
        assert_eq!(model().flatten_visible().len(), 2);
    }

    #[test]
    fn move_cursor_down() {
        let mut model = model();
        model.move_cursor(1);
        assert_eq!(model.cursor, 1);
    }

    #[test]
    fn move_cursor_saturates() {
        let mut model = model();
        model.move_cursor(-1);
        assert_eq!(model.cursor, 0);
    }

    #[test]
    fn toggle_expand_hides_children() {
        let mut model = model();
        model.toggle_expand();
        assert_eq!(model.flatten_visible().len(), 1);
    }

    #[test]
    fn selected_returns_cursor_node() {
        let mut model = model();
        model.move_cursor(1);
        assert_eq!(model.selected().unwrap().label, "child");
    }

    #[test]
    fn default_sort_places_highest_consumed_dir_group_first() {
        let tree = build_tree(
            ViewMode::DirsDuplicated,
            &[dir_group("small_many", 10, 5), dir_group("big_few", 30, 2)],
            &[],
            SortConfig::default(),
            Path::new("root"),
        );

        assert_eq!(tree.roots[0].children[0].path, PathBuf::from("big_few/0"));
    }

    #[test]
    fn file_count_sort_uses_group_repetition_count() {
        let tree = build_tree(
            ViewMode::DirsDuplicated,
            &[dir_group("few", 100, 2), dir_group("many", 1, 4)],
            &[],
            SortConfig {
                by: SortBy::FileCount,
                order: SortOrder::Desc,
            },
            Path::new("root"),
        );

        assert_eq!(tree.roots[0].children[0].path, PathBuf::from("many/0"));
    }

    fn dir_group(name: &str, size_bytes: u64, copies: usize) -> DupGroup {
        DupGroup {
            id: Some(1),
            dir_hash: DirHash::new(format!("hash-{name}")),
            file_count: 1,
            size_bytes,
            entries: (0..copies)
                .map(|index| DupEntry {
                    id: Some(i64::try_from(index + 1).unwrap()),
                    path: PathBuf::from(format!("{name}/{index}")),
                    status: EntryStatus::Pending,
                    dir_hash: DirHash::new(format!("hash-{name}")),
                })
                .collect(),
        }
    }
}
