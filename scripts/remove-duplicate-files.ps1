param(
    [string]$Root = "work/ddup-e2e/root",
    [string]$DbPath = "work/ddup-e2e/ddup-test.sqlite"
)

$ErrorActionPreference = "Stop"
$rootPath = Resolve-Path -LiteralPath $Root
$dbFullPath = Join-Path (Resolve-Path -LiteralPath ".") $DbPath

New-Item -ItemType Directory -Force -Path (Split-Path -Parent $dbFullPath) | Out-Null

cargo run -q -p ddup -- $rootPath --db $dbFullPath --no-tui --mode flat
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$script = @'
import os
import sqlite3
import sys
from pathlib import Path

root = Path(sys.argv[1]).resolve()
db_path = Path(sys.argv[2]).resolve()
conn = sqlite3.connect(db_path)
rows = conn.execute("""
    SELECT dfe.id, dfe.group_id, dfe.path
    FROM dup_file_entries dfe
    JOIN dup_file_groups dfg ON dfg.id = dfe.group_id
    ORDER BY dfe.group_id, dfe.path
""").fetchall()

groups = {}
for entry_id, group_id, path in rows:
    groups.setdefault(group_id, []).append((entry_id, Path(path).resolve()))

deleted = []
for entries in groups.values():
    if len(entries) < 2:
        continue
    keep_id, keep_path = entries[0]
    conn.execute("UPDATE dup_file_entries SET status = 'kept' WHERE id = ?", (keep_id,))
    for entry_id, path in entries[1:]:
        if os.path.commonpath([root, path]) != str(root):
            raise RuntimeError(f"Refusing to delete outside root: {path}")
        if path.exists():
            path.unlink()
            deleted.append(str(path))
        conn.execute("UPDATE dup_file_entries SET status = 'deleted' WHERE id = ?", (entry_id,))

conn.commit()
conn.close()
print(f"Deleted files: {len(deleted)}")
for path in deleted:
    print(path)
'@

python -c $script $rootPath $dbFullPath
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
