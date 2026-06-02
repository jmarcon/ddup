//! TUI tree model.

use std::path::PathBuf;

use ddup_core::{DupFileGroup, DupGroup, SortConfig};

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
    /// Display label.
    pub label: String,
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
    _sort: SortConfig,
) -> TreeModel {
    let roots = match view {
        ViewMode::DirsDuplicated => dir_groups
            .iter()
            .map(|group| TreeNode {
                path: PathBuf::new(),
                label: format!("Dir group {}", group.id.unwrap_or_default()),
                kind: NodeKind::GroupRoot,
                expanded: true,
                group_id: group.id,
                dup_marker: "dir".to_owned(),
                children: group
                    .entries
                    .iter()
                    .map(|entry| TreeNode {
                        path: entry.path.clone(),
                        label: entry.path.display().to_string(),
                        kind: NodeKind::DirEntry,
                        expanded: false,
                        children: Vec::new(),
                        group_id: group.id,
                        dup_marker: entry.status.to_string(),
                    })
                    .collect(),
            })
            .collect(),
        ViewMode::FilesDuplicatedSmart | ViewMode::FilesDuplicatedFlat => file_groups
            .iter()
            .map(|group| TreeNode {
                path: PathBuf::new(),
                label: format!("File group {}", group.id.unwrap_or_default()),
                kind: NodeKind::GroupRoot,
                expanded: true,
                group_id: group.id,
                dup_marker: "file".to_owned(),
                children: group
                    .entries
                    .iter()
                    .map(|entry| TreeNode {
                        path: entry.path.clone(),
                        label: entry.path.display().to_string(),
                        kind: NodeKind::FileEntry,
                        expanded: false,
                        children: Vec::new(),
                        group_id: group.id,
                        dup_marker: entry.status.to_string(),
                    })
                    .collect(),
            })
            .collect(),
    };
    TreeModel { roots, cursor: 0 }
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

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

    use super::*;

    fn model() -> TreeModel {
        TreeModel {
            cursor: 0,
            roots: vec![TreeNode {
                path: PathBuf::from("root"),
                label: "root".to_owned(),
                kind: NodeKind::GroupRoot,
                expanded: true,
                children: vec![TreeNode {
                    path: PathBuf::from("child"),
                    label: "child".to_owned(),
                    kind: NodeKind::DirEntry,
                    expanded: false,
                    children: Vec::new(),
                    group_id: Some(1),
                    dup_marker: "pending".to_owned(),
                }],
                group_id: Some(1),
                dup_marker: "dir".to_owned(),
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
}
