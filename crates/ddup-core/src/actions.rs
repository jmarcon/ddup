//! Filesystem actions for duplicate entries.

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{CoreError, Db, EntryStatus, Result};

/// Opens a path in the platform file manager.
pub fn open_in_explorer(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(CoreError::PathNotFound(path.to_path_buf()));
    }
    open::that(path).map_err(|error| CoreError::InvalidState(error.to_string()))?;
    Ok(())
}

/// Deletes a directory duplicate entry and marks it deleted.
pub fn delete_entry(db: &Db, entry_id: i64) -> Result<()> {
    let entry = db.fetch_entry(entry_id)?;
    if !entry.path.exists() {
        return Err(CoreError::PathNotFound(entry.path));
    }
    fs::remove_dir_all(&entry.path).map_err(|source| CoreError::Io {
        path: entry.path.clone(),
        source,
    })?;
    db.update_entry_status(entry_id, EntryStatus::Deleted)
}

/// Moves a directory duplicate entry and marks it moved.
pub fn move_entry(db: &Db, entry_id: i64, dest: &Path) -> Result<()> {
    let entry = db.fetch_entry(entry_id)?;
    move_path(&entry.path, dest, true)?;
    db.update_entry_status(entry_id, EntryStatus::Moved)
}

/// Deletes a file duplicate entry and marks it deleted.
pub fn delete_file_entry(db: &Db, entry_id: i64) -> Result<()> {
    let entry = db.fetch_file_entry(entry_id)?;
    if !entry.path.exists() {
        return Err(CoreError::PathNotFound(entry.path));
    }
    fs::remove_file(&entry.path).map_err(|source| CoreError::Io {
        path: entry.path.clone(),
        source,
    })?;
    db.update_file_entry_status(entry_id, EntryStatus::Deleted)
}

/// Moves a file duplicate entry and marks it moved.
pub fn move_file_entry(db: &Db, entry_id: i64, dest: &Path) -> Result<()> {
    let entry = db.fetch_file_entry(entry_id)?;
    move_path(&entry.path, dest, false)?;
    db.update_file_entry_status(entry_id, EntryStatus::Moved)
}

fn move_path(source: &Path, dest: &Path, is_dir: bool) -> Result<()> {
    if !source.exists() {
        return Err(CoreError::PathNotFound(source.to_path_buf()));
    }
    if dest.exists() {
        return Err(CoreError::InvalidState(format!(
            "destination exists: {}",
            dest.display()
        )));
    }
    match fs::rename(source, dest) {
        Ok(()) => Ok(()),
        Err(error) => {
            if is_dir {
                copy_dir_all(source, dest)?;
                fs::remove_dir_all(source).map_err(|source_error| CoreError::Io {
                    path: source.to_path_buf(),
                    source: source_error,
                })?;
            } else {
                fs::copy(source, dest).map_err(|source_error| CoreError::Io {
                    path: source.to_path_buf(),
                    source: source_error,
                })?;
                fs::remove_file(source).map_err(|source_error| CoreError::Io {
                    path: source.to_path_buf(),
                    source: source_error,
                })?;
            }
            let _ = error;
            Ok(())
        }
    }
}

fn copy_dir_all(source: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest).map_err(|error| CoreError::Io {
        path: dest.to_path_buf(),
        source: error,
    })?;
    for entry in fs::read_dir(source).map_err(|error| CoreError::Io {
        path: source.to_path_buf(),
        source: error,
    })? {
        let entry = entry.map_err(|error| CoreError::Io {
            path: source.to_path_buf(),
            source: error,
        })?;
        let from = entry.path();
        let to = PathBuf::from(dest).join(entry.file_name());
        if from.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(|error| CoreError::Io {
                path: from,
                source: error,
            })?;
        }
    }
    Ok(())
}
