//! SQLite persistence.

use std::{path::Path, str::FromStr};

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};

use crate::{
    DirHash, DupEntry, DupFileEntry, DupFileGroup, DupGroup, DupStatus, EntryStatus, FileHash,
    NodeKind, Result, Scan, ScanMode, SortBy, SortConfig, SortOrder, TreeStats,
};

const SCHEMA: &str = include_str!("schema_v2.sql");

/// SQLite database handle.
pub struct Db {
    conn: Connection,
}

impl Db {
    /// Opens a database and migrates any pre-v2 database by recreating the v2 schema.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        migrate(&conn)?;
        Ok(Self { conn })
    }

    /// Opens an in-memory database.
    pub fn memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        migrate(&conn)?;
        Ok(Self { conn })
    }

    /// Inserts or replaces a scan by root path.
    pub fn upsert_scan(&self, scan: &Scan) -> Result<i64> {
        self.conn.execute(
            "INSERT OR REPLACE INTO scans
             (root_path, scanned_at, scan_mode, total_dirs, total_files, wasted_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                path_to_string(&scan.root_path),
                scan.scanned_at.to_rfc3339(),
                scan.scan_mode.to_string(),
                scan.total_dirs,
                scan.total_files,
                scan.wasted_bytes
            ],
        )?;
        Ok(self.conn.query_row(
            "SELECT id FROM scans WHERE root_path = ?1",
            [path_to_string(&scan.root_path)],
            |row| row.get(0),
        )?)
    }

    /// Fetches a scan by root path.
    pub fn fetch_scan(&self, root_path: &Path) -> Result<Option<Scan>> {
        self.conn
            .query_row(
                "SELECT id, root_path, scanned_at, scan_mode, total_dirs, total_files, wasted_bytes
                 FROM scans WHERE root_path = ?1",
                [path_to_string(root_path)],
                row_to_scan,
            )
            .optional()
            .map_err(Into::into)
    }

    /// Lists scans ordered by id.
    pub fn list_scans(&self) -> Result<Vec<Scan>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, root_path, scanned_at, scan_mode, total_dirs, total_files, wasted_bytes
             FROM scans ORDER BY id",
        )?;
        collect_rows(stmt.query_map([], row_to_scan)?)
    }

    /// Fetches latest scan.
    pub fn last_scan(&self) -> Result<Option<Scan>> {
        self.conn
            .query_row(
                "SELECT id, root_path, scanned_at, scan_mode, total_dirs, total_files, wasted_bytes
                 FROM scans ORDER BY id DESC LIMIT 1",
                [],
                row_to_scan,
            )
            .optional()
            .map_err(Into::into)
    }

    /// Deletes a scan.
    pub fn delete_scan(&self, scan_id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM scans WHERE id = ?1", [scan_id])?;
        Ok(())
    }

    /// Inserts directory duplicate groups.
    pub fn insert_dir_groups(&mut self, scan_id: i64, groups: &[DupGroup]) -> Result<()> {
        let tx = self.conn.transaction()?;
        for group in groups {
            tx.execute(
                "INSERT INTO dup_groups (scan_id, dir_hash, file_count, size_bytes)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    scan_id,
                    group.dir_hash.as_str(),
                    group.file_count,
                    group.size_bytes
                ],
            )?;
            let group_id = tx.last_insert_rowid();
            for entry in &group.entries {
                tx.execute(
                    "INSERT INTO dup_entries (group_id, path, status, dir_hash)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![
                        group_id,
                        path_to_string(&entry.path),
                        entry.status.to_string(),
                        entry.dir_hash.as_str()
                    ],
                )?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Fetches directory duplicate groups.
    pub fn fetch_dir_groups(&self, scan_id: i64, sort: SortConfig) -> Result<Vec<DupGroup>> {
        let mut groups = self.fetch_dir_groups_unsorted(scan_id)?;
        sort_dir_groups(&mut groups, sort);
        Ok(groups)
    }

    /// Inserts file duplicate groups.
    pub fn insert_file_groups(&mut self, scan_id: i64, groups: &[DupFileGroup]) -> Result<()> {
        let tx = self.conn.transaction()?;
        for group in groups {
            tx.execute(
                "INSERT INTO dup_file_groups (scan_id, file_hash, size_bytes)
                 VALUES (?1, ?2, ?3)",
                params![scan_id, group.file_hash.as_str(), group.size_bytes],
            )?;
            let group_id = tx.last_insert_rowid();
            for entry in &group.entries {
                tx.execute(
                    "INSERT INTO dup_file_entries
                     (group_id, path, status, suppressed_by_dir_group)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![
                        group_id,
                        path_to_string(&entry.path),
                        entry.status.to_string(),
                        entry.suppressed_by_dir_group
                    ],
                )?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Fetches file duplicate groups.
    pub fn fetch_file_groups(
        &self,
        scan_id: i64,
        sort: SortConfig,
        filter_suppressed: bool,
    ) -> Result<Vec<DupFileGroup>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, file_hash, size_bytes FROM dup_file_groups WHERE scan_id = ?1")?;
        let ids = collect_rows(stmt.query_map([scan_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, u64>(2)?,
            ))
        })?)?;
        let mut groups = Vec::new();
        for (id, hash, size_bytes) in ids {
            let entries = self.fetch_file_entries(id, filter_suppressed)?;
            if entries.len() >= 2 || !filter_suppressed {
                groups.push(DupFileGroup {
                    id: Some(id),
                    file_hash: FileHash::new(hash),
                    size_bytes,
                    entries,
                });
            }
        }
        sort_file_groups(&mut groups, sort);
        Ok(groups)
    }

    /// Inserts tree nodes.
    pub fn insert_tree_nodes(&mut self, scan_id: i64, nodes: &[TreeStats]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO tree_nodes
                 (scan_id, path, parent_path, kind, depth, size_bytes, size_recursive,
                  file_count_recursive, extension, dup_status, dir_group_id, file_group_id,
                  wasted_bytes, content_hash)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            )?;
            for node in nodes {
                stmt.execute(params![
                    scan_id,
                    path_to_string(&node.path),
                    node.parent_path.as_deref().map(path_to_string),
                    node_kind_to_string(node.kind),
                    node.depth,
                    node.size_bytes,
                    node.size_recursive,
                    node.file_count_recursive,
                    node.extension,
                    node.dup_status.to_string(),
                    node.dir_group_id,
                    node.file_group_id,
                    node.wasted_bytes,
                    node.content_hash
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Fetches a single tree node.
    pub fn fetch_node(&self, scan_id: i64, path: &Path) -> Result<Option<TreeStats>> {
        self.conn
            .query_row(
                "SELECT path, parent_path, kind, depth, size_bytes, size_recursive,
                        file_count_recursive, extension, dup_status, dir_group_id,
                        file_group_id, wasted_bytes, content_hash
                 FROM tree_nodes WHERE scan_id = ?1 AND path = ?2",
                params![scan_id, path_to_string(path)],
                row_to_tree,
            )
            .optional()
            .map_err(Into::into)
    }

    /// Fetches direct children for GUI lazy loading.
    pub fn fetch_children(&self, scan_id: i64, parent_path: &Path) -> Result<Vec<TreeStats>> {
        let mut stmt = self.conn.prepare(
            "SELECT path, parent_path, kind, depth, size_bytes, size_recursive,
                    file_count_recursive, extension, dup_status, dir_group_id,
                    file_group_id, wasted_bytes, content_hash
             FROM tree_nodes WHERE scan_id = ?1 AND parent_path = ?2 ORDER BY path",
        )?;
        collect_rows(stmt.query_map(params![scan_id, path_to_string(parent_path)], row_to_tree)?)
    }

    /// Fetches a subtree for treemap or sunburst rendering.
    pub fn fetch_subtree(
        &self,
        scan_id: i64,
        root: &Path,
        max_depth: Option<u32>,
    ) -> Result<Vec<TreeStats>> {
        let root_string = path_to_string(root);
        let root_depth = self.fetch_node(scan_id, root)?.map_or(0, |node| node.depth);
        let all = self.fetch_subtree_filtered(scan_id, root, false)?;
        Ok(all
            .into_iter()
            .filter(|node| {
                max_depth.is_none_or(|max| node.depth.saturating_sub(root_depth) <= max)
                    && path_to_string(&node.path).starts_with(&root_string)
            })
            .collect())
    }

    /// Fetches a subtree optionally limited to nodes with waste.
    pub fn fetch_subtree_filtered(
        &self,
        scan_id: i64,
        root: &Path,
        only_with_waste: bool,
    ) -> Result<Vec<TreeStats>> {
        let mut stmt = self.conn.prepare(
            "SELECT path, parent_path, kind, depth, size_bytes, size_recursive,
                    file_count_recursive, extension, dup_status, dir_group_id,
                    file_group_id, wasted_bytes, content_hash
             FROM tree_nodes WHERE scan_id = ?1 AND path LIKE ?2 ORDER BY path",
        )?;
        let pattern = format!("{}%", path_to_string(root));
        let rows = collect_rows(stmt.query_map(params![scan_id, pattern], row_to_tree)?)?;
        Ok(rows
            .into_iter()
            .filter(|node| !only_with_waste || node.wasted_bytes > 0)
            .collect())
    }

    /// Updates a directory entry status.
    pub fn update_entry_status(&self, entry_id: i64, status: EntryStatus) -> Result<()> {
        self.conn.execute(
            "UPDATE dup_entries SET status = ?1 WHERE id = ?2",
            params![status.to_string(), entry_id],
        )?;
        Ok(())
    }

    /// Updates a file entry status.
    pub fn update_file_entry_status(&self, entry_id: i64, status: EntryStatus) -> Result<()> {
        self.conn.execute(
            "UPDATE dup_file_entries SET status = ?1 WHERE id = ?2",
            params![status.to_string(), entry_id],
        )?;
        Ok(())
    }

    /// Fetches a directory entry by id.
    pub fn fetch_entry(&self, entry_id: i64) -> Result<DupEntry> {
        self.conn
            .query_row(
                "SELECT id, path, status, dir_hash FROM dup_entries WHERE id = ?1",
                [entry_id],
                row_to_dup_entry,
            )
            .map_err(Into::into)
    }

    /// Fetches a file entry by id.
    pub fn fetch_file_entry(&self, entry_id: i64) -> Result<DupFileEntry> {
        self.conn.query_row(
            "SELECT id, path, status, suppressed_by_dir_group FROM dup_file_entries WHERE id = ?1",
            [entry_id],
            row_to_dup_file_entry,
        ).map_err(Into::into)
    }

    fn fetch_dir_groups_unsorted(&self, scan_id: i64) -> Result<Vec<DupGroup>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, dir_hash, file_count, size_bytes FROM dup_groups WHERE scan_id = ?1",
        )?;
        let ids = collect_rows(stmt.query_map([scan_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, u64>(2)?,
                row.get::<_, u64>(3)?,
            ))
        })?)?;
        let mut groups = Vec::new();
        for (id, hash, file_count, size_bytes) in ids {
            let mut entry_stmt = self.conn.prepare(
                "SELECT id, path, status, dir_hash FROM dup_entries WHERE group_id = ?1 ORDER BY path",
            )?;
            let entries = collect_rows(entry_stmt.query_map([id], row_to_dup_entry)?)?;
            groups.push(DupGroup {
                id: Some(id),
                dir_hash: DirHash::new(hash),
                file_count,
                size_bytes,
                entries,
            });
        }
        Ok(groups)
    }

    fn fetch_file_entries(
        &self,
        group_id: i64,
        filter_suppressed: bool,
    ) -> Result<Vec<DupFileEntry>> {
        let sql = if filter_suppressed {
            "SELECT id, path, status, suppressed_by_dir_group
             FROM dup_file_entries
             WHERE group_id = ?1 AND suppressed_by_dir_group IS NULL ORDER BY path"
        } else {
            "SELECT id, path, status, suppressed_by_dir_group
             FROM dup_file_entries WHERE group_id = ?1 ORDER BY path"
        };
        let mut stmt = self.conn.prepare(sql)?;
        collect_rows(stmt.query_map([group_id], row_to_dup_file_entry)?)
    }
}

fn migrate(conn: &Connection) -> Result<()> {
    let version = conn.pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))?;
    if version < 2 {
        conn.execute_batch(SCHEMA)?;
        return Ok(());
    }
    if version < 3 {
        conn.execute("ALTER TABLE tree_nodes ADD COLUMN content_hash TEXT", [])?;
        conn.pragma_update(None, "user_version", 3)?;
    }
    Ok(())
}

fn collect_rows<T>(rows: impl Iterator<Item = rusqlite::Result<T>>) -> Result<Vec<T>> {
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn row_to_scan(row: &rusqlite::Row<'_>) -> rusqlite::Result<Scan> {
    let scanned_at: String = row.get(2)?;
    let scan_mode: String = row.get(3)?;
    Ok(Scan {
        id: Some(row.get(0)?),
        root_path: row.get::<_, String>(1)?.into(),
        scanned_at: DateTime::parse_from_rfc3339(&scanned_at)
            .map(|value| value.with_timezone(&Utc))
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?,
        scan_mode: ScanMode::from_str(&scan_mode)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?,
        total_dirs: row.get(4)?,
        total_files: row.get(5)?,
        wasted_bytes: row.get(6)?,
    })
}

fn row_to_dup_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<DupEntry> {
    let status: String = row.get(2)?;
    Ok(DupEntry {
        id: Some(row.get(0)?),
        path: row.get::<_, String>(1)?.into(),
        status: EntryStatus::from_str(&status)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?,
        dir_hash: DirHash::new(row.get::<_, String>(3)?),
    })
}

fn row_to_dup_file_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<DupFileEntry> {
    let status: String = row.get(2)?;
    Ok(DupFileEntry {
        id: Some(row.get(0)?),
        path: row.get::<_, String>(1)?.into(),
        status: EntryStatus::from_str(&status)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?,
        suppressed_by_dir_group: row.get(3)?,
    })
}

fn row_to_tree(row: &rusqlite::Row<'_>) -> rusqlite::Result<TreeStats> {
    let kind: String = row.get(2)?;
    let dup_status: String = row.get(8)?;
    Ok(TreeStats {
        path: row.get::<_, String>(0)?.into(),
        parent_path: row.get::<_, Option<String>>(1)?.map(Into::into),
        kind: parse_node_kind(&kind)?,
        depth: row.get(3)?,
        size_bytes: row.get(4)?,
        size_recursive: row.get(5)?,
        file_count_recursive: row.get(6)?,
        extension: row.get(7)?,
        dup_status: DupStatus::from_str(&dup_status)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?,
        dir_group_id: row.get(9)?,
        file_group_id: row.get(10)?,
        wasted_bytes: row.get(11)?,
        content_hash: row.get(12)?,
    })
}

fn node_kind_to_string(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Dir => "dir",
        NodeKind::File => "file",
    }
}

fn parse_node_kind(value: &str) -> rusqlite::Result<NodeKind> {
    match value {
        "dir" => Ok(NodeKind::Dir),
        "file" => Ok(NodeKind::File),
        other => Err(rusqlite::Error::ToSqlConversionFailure(Box::new(
            crate::CoreError::InvalidState(format!("unknown node kind: {other}")),
        ))),
    }
}

fn sort_dir_groups(groups: &mut [DupGroup], sort: SortConfig) {
    groups.sort_by(|a, b| match sort.by {
        SortBy::Name => a.entries[0].path.cmp(&b.entries[0].path),
        SortBy::TotalSize => a.size_bytes.cmp(&b.size_bytes),
        SortBy::FileCount => a.file_count.cmp(&b.file_count),
    });
    if sort.order == SortOrder::Desc {
        groups.reverse();
    }
}

fn sort_file_groups(groups: &mut [DupFileGroup], sort: SortConfig) {
    groups.sort_by(|a, b| match sort.by {
        SortBy::Name => a.entries[0].path.cmp(&b.entries[0].path),
        SortBy::TotalSize => a.size_bytes.cmp(&b.size_bytes),
        SortBy::FileCount => a.entries.len().cmp(&b.entries.len()),
    });
    if sort.order == SortOrder::Desc {
        groups.reverse();
    }
}
