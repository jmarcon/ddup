param(
    [string]$Root = "work/ddup-e2e/root",
    [string]$DbPath = "work/ddup-e2e/ddup-test.sqlite",
    [string]$LocalAppData = "work/ddup-e2e/localappdata"
)

$ErrorActionPreference = "Stop"

function Invoke-Step {
    param(
        [string]$Name,
        [scriptblock]$Body
    )

    Write-Host "== $Name"
    & $Body
}

function Assert-SqliteHasScan {
    param(
        [string]$Path
    )

    $script = @'
import sqlite3
import sys
from pathlib import Path

db_path = Path(sys.argv[1])
if not db_path.exists():
    raise SystemExit(f"missing db: {db_path}")

conn = sqlite3.connect(db_path)
scan_count = conn.execute("SELECT COUNT(*) FROM scans").fetchone()[0]
scan_id = conn.execute("SELECT id FROM scans ORDER BY id DESC LIMIT 1").fetchone()
if scan_id is None:
    raise SystemExit("expected at least one scan")
scan_id = scan_id[0]
dir_group_count = conn.execute("SELECT COUNT(*) FROM dup_groups WHERE scan_id = ?", (scan_id,)).fetchone()[0]
file_group_count = conn.execute("SELECT COUNT(*) FROM dup_file_groups WHERE scan_id = ?", (scan_id,)).fetchone()[0]
tree_count = conn.execute("SELECT COUNT(*) FROM tree_nodes WHERE scan_id = ?", (scan_id,)).fetchone()[0]
missing_hash_count = conn.execute("SELECT COUNT(*) FROM tree_nodes WHERE scan_id = ? AND content_hash IS NULL", (scan_id,)).fetchone()[0]
conn.close()

if dir_group_count < 1:
    raise SystemExit("expected duplicate directory groups")
if file_group_count < 1:
    raise SystemExit("expected duplicate file groups")
if tree_count < 1:
    raise SystemExit("expected tree nodes")
if missing_hash_count != 0:
    raise SystemExit(f"expected every tree node to have content_hash, missing={missing_hash_count}")

print(f"SQLite ok: scans={scan_count}, dir_groups={dir_group_count}, file_groups={file_group_count}, tree_nodes={tree_count}, missing_hashes={missing_hash_count}")
'@

    python -c $script $Path
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

Invoke-Step "clean" {
    ./scripts/clean-test-fixtures.ps1
}

Invoke-Step "create fixtures" {
    ./scripts/new-test-fixtures.ps1 -Root $Root
}

Invoke-Step "help exposes e2e flags" {
    $help = (cargo run -q -p ddup -- --help) -join "`n"
    if ($help -notmatch "--no-tui") { throw "missing --no-tui in help" }
    if ($help -notmatch "--no-icons") { throw "missing --no-icons in help" }
    if ($help -notmatch "--db") { throw "missing --db in help" }
    if ($help -notmatch "--mode") { throw "missing --mode in help" }
}

Invoke-Step "default sqlite path persists" {
    $oldDataLocal = $env:DDUP_DATA_LOCAL_DIR
    $env:DDUP_DATA_LOCAL_DIR = Join-Path (Resolve-Path -LiteralPath ".") $LocalAppData
    try {
        $output = cargo run -q -p ddup -- (Resolve-Path -LiteralPath $Root) --no-tui --mode smart
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        $text = $output -join "`n"
        if ($text -notmatch "SQLite:\s*(.+)") {
            throw "missing SQLite path in output"
        }
        Assert-SqliteHasScan -Path $Matches[1].Trim()
    } finally {
        $env:DDUP_DATA_LOCAL_DIR = $oldDataLocal
    }
}

Invoke-Step "custom sqlite persists" {
    cargo run -q -p ddup -- (Resolve-Path -LiteralPath $Root) --db (Join-Path (Resolve-Path -LiteralPath ".") $DbPath) --no-tui --mode smart
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    Assert-SqliteHasScan -Path (Join-Path (Resolve-Path -LiteralPath ".") $DbPath)
}

Invoke-Step "multi-root compares between roots" {
    $base = Join-Path (Resolve-Path -LiteralPath ".") "work/ddup-e2e/multi"
    $left = Join-Path $base "left"
    $right = Join-Path $base "right"
    $multiDb = Join-Path $base "multi.sqlite"
    New-Item -ItemType Directory -Force -Path (Join-Path $left "cross") | Out-Null
    New-Item -ItemType Directory -Force -Path (Join-Path $right "cross") | Out-Null
    Set-Content -LiteralPath (Join-Path $left "cross/a.txt") -Value "cross" -NoNewline
    Set-Content -LiteralPath (Join-Path $right "cross/a.txt") -Value "cross" -NoNewline
    Set-Content -LiteralPath (Join-Path $left "local-a.txt") -Value "local" -NoNewline
    Set-Content -LiteralPath (Join-Path $left "local-b.txt") -Value "local" -NoNewline

    cargo run -q -p ddup -- $left $right --db $multiDb --no-tui --mode flat
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    $script = @'
import sqlite3
import sys
from pathlib import Path

db_path = Path(sys.argv[1])
left = str(Path(sys.argv[2]).resolve())
right = str(Path(sys.argv[3]).resolve())

conn = sqlite3.connect(db_path)
scan_id = conn.execute("SELECT id FROM scans ORDER BY id DESC LIMIT 1").fetchone()[0]
rows = conn.execute("""
SELECT g.id, e.path
FROM dup_groups g
JOIN dup_entries e ON e.group_id = g.id
WHERE g.scan_id = ?
""", (scan_id,)).fetchall()
file_rows = conn.execute("""
SELECT g.id, e.path
FROM dup_file_groups g
JOIN dup_file_entries e ON e.group_id = g.id
WHERE g.scan_id = ?
""", (scan_id,)).fetchall()
conn.close()

def spans(rows):
    groups = {}
    for group_id, path in rows:
        groups.setdefault(group_id, []).append(str(Path(path).resolve()))
    return any(any(p.startswith(left) for p in paths) and any(p.startswith(right) for p in paths) for paths in groups.values())

def local_only(rows):
    groups = {}
    for group_id, path in rows:
        groups.setdefault(group_id, []).append(str(Path(path).resolve()))
    return any(all(p.startswith(left) for p in paths) for paths in groups.values())

if not spans(rows):
    raise SystemExit("expected directory group spanning both roots")
if not spans(file_rows):
    raise SystemExit("expected file group spanning both roots")
if local_only(file_rows):
    raise SystemExit("local-only duplicate file group should be ignored")
print("Multi-root ok")
'@
    python -c $script $multiDb $left $right
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

Invoke-Step "remove duplicate directories" {
    ./scripts/remove-duplicate-dirs.ps1 -Root $Root -DbPath $DbPath
}

Invoke-Step "validate duplicate directories removed" {
    ./scripts/validate-duplicate-dirs-removed.ps1 -Root $Root
}

Invoke-Step "remove duplicate files" {
    ./scripts/remove-duplicate-files.ps1 -Root $Root -DbPath $DbPath
}

Invoke-Step "validate duplicate files removed" {
    ./scripts/validate-duplicate-files-removed.ps1 -Root $Root
}

Invoke-Step "cleanup" {
    ./scripts/clean-test-fixtures.ps1
}

Write-Host "E2E passed."
