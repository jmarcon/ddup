param(
    [string]$Root = "work/ddup-e2e/root",
    [string]$DbPath = "work/ddup-e2e/ddup-test.sqlite"
)

$ErrorActionPreference = "Stop"
$rootPath = Resolve-Path -LiteralPath $Root
$dbFullPath = Join-Path (Resolve-Path -LiteralPath ".") $DbPath

New-Item -ItemType Directory -Force -Path (Split-Path -Parent $dbFullPath) | Out-Null

cargo run -q -p ddup -- $rootPath --db $dbFullPath --no-tui --mode smart
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$script = @'
import os
import shutil
import sqlite3
import sys
from pathlib import Path

root = Path(sys.argv[1]).resolve()
db_path = Path(sys.argv[2]).resolve()
conn = sqlite3.connect(db_path)
rows = conn.execute("""
    SELECT de.id, de.group_id, de.path
    FROM dup_entries de
    JOIN dup_groups dg ON dg.id = de.group_id
    ORDER BY de.group_id, de.path
""").fetchall()

groups = {}
for entry_id, group_id, path in rows:
    groups.setdefault(group_id, []).append((entry_id, Path(path).resolve()))

deleted = []
for entries in groups.values():
    if len(entries) < 2:
        continue
    keep_id, keep_path = entries[0]
    conn.execute("UPDATE dup_entries SET status = 'kept' WHERE id = ?", (keep_id,))
    for entry_id, path in entries[1:]:
        if os.path.commonpath([root, path]) != str(root):
            raise RuntimeError(f"Refusing to delete outside root: {path}")
        if path.exists():
            shutil.rmtree(path)
            deleted.append(str(path))
        conn.execute("UPDATE dup_entries SET status = 'deleted' WHERE id = ?", (entry_id,))

conn.commit()
conn.close()
print(f"Deleted directories: {len(deleted)}")
for path in deleted:
    print(path)
'@

python -c $script $rootPath $dbFullPath
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
