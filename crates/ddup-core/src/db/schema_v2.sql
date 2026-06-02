PRAGMA user_version = 2;

DROP TABLE IF EXISTS tree_nodes;
DROP TABLE IF EXISTS dup_file_entries;
DROP TABLE IF EXISTS dup_file_groups;
DROP TABLE IF EXISTS dup_entries;
DROP TABLE IF EXISTS dup_groups;
DROP TABLE IF EXISTS scans;

CREATE TABLE scans (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  root_path     TEXT    NOT NULL UNIQUE,
  scanned_at    TEXT    NOT NULL,
  scan_mode     TEXT    NOT NULL DEFAULT 'smart',
  total_dirs    INTEGER NOT NULL DEFAULT 0,
  total_files   INTEGER NOT NULL DEFAULT 0,
  wasted_bytes  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE dup_groups (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  scan_id       INTEGER NOT NULL REFERENCES scans(id) ON DELETE CASCADE,
  dir_hash      TEXT    NOT NULL,
  file_count    INTEGER NOT NULL,
  size_bytes    INTEGER NOT NULL
);

CREATE TABLE dup_entries (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  group_id      INTEGER NOT NULL REFERENCES dup_groups(id) ON DELETE CASCADE,
  path          TEXT    NOT NULL,
  status        TEXT    NOT NULL DEFAULT 'pending',
  dir_hash      TEXT
);

CREATE TABLE dup_file_groups (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  scan_id       INTEGER NOT NULL REFERENCES scans(id) ON DELETE CASCADE,
  file_hash     TEXT    NOT NULL,
  size_bytes    INTEGER NOT NULL
);

CREATE TABLE dup_file_entries (
  id                       INTEGER PRIMARY KEY AUTOINCREMENT,
  group_id                 INTEGER NOT NULL REFERENCES dup_file_groups(id) ON DELETE CASCADE,
  path                     TEXT    NOT NULL,
  status                   TEXT    NOT NULL DEFAULT 'pending',
  suppressed_by_dir_group  INTEGER REFERENCES dup_groups(id) ON DELETE SET NULL
);

CREATE TABLE tree_nodes (
  id                    INTEGER PRIMARY KEY AUTOINCREMENT,
  scan_id               INTEGER NOT NULL REFERENCES scans(id) ON DELETE CASCADE,
  path                  TEXT    NOT NULL,
  parent_path           TEXT,
  kind                  TEXT    NOT NULL,
  depth                 INTEGER NOT NULL,
  size_bytes            INTEGER NOT NULL,
  size_recursive        INTEGER NOT NULL,
  file_count_recursive  INTEGER NOT NULL,
  extension             TEXT,
  dup_status            TEXT    NOT NULL,
  dir_group_id          INTEGER REFERENCES dup_groups(id) ON DELETE SET NULL,
  file_group_id         INTEGER REFERENCES dup_file_groups(id) ON DELETE SET NULL,
  wasted_bytes          INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_dup_groups_scan         ON dup_groups(scan_id);
CREATE INDEX idx_dup_entries_group       ON dup_entries(group_id);
CREATE INDEX idx_dup_groups_hash         ON dup_groups(dir_hash);
CREATE INDEX idx_dup_file_groups_scan    ON dup_file_groups(scan_id);
CREATE INDEX idx_dup_file_entries_group  ON dup_file_entries(group_id);
CREATE INDEX idx_dup_file_groups_hash    ON dup_file_groups(file_hash);
CREATE INDEX idx_tree_nodes_scan         ON tree_nodes(scan_id);
CREATE INDEX idx_tree_nodes_parent       ON tree_nodes(scan_id, parent_path);
CREATE INDEX idx_tree_nodes_dup_status   ON tree_nodes(scan_id, dup_status);
CREATE INDEX idx_tree_nodes_path         ON tree_nodes(scan_id, path);

