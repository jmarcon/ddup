param(
    [string]$Root = "work/ddup-e2e/root"
)

$ErrorActionPreference = "Stop"
$rootPath = Resolve-Path -LiteralPath $Root

$mustExist = @(
    "dirs/a_keep_dir",
    "dirs/unique_dir",
    "files/a_keep_file/report.txt",
    "files/b_delete_file/report-copy.txt",
    "files/c_delete_file/report-copy.txt"
)

$mustNotExist = @(
    "dirs/b_delete_dir",
    "dirs/c_delete_dir"
)

foreach ($path in $mustExist) {
    $full = Join-Path $rootPath $path
    if (-not (Test-Path -LiteralPath $full)) { throw "Missing expected path: $full" }
}

foreach ($path in $mustNotExist) {
    $full = Join-Path $rootPath $path
    if (Test-Path -LiteralPath $full) { throw "Unexpected path still exists: $full" }
}

Write-Host "Directory cleanup validation passed."
