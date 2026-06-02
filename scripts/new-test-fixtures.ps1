param(
    [string]$Root = "work/ddup-e2e/root",
    [switch]$Force
)

$ErrorActionPreference = "Stop"
$rootPath = Resolve-Path -LiteralPath "." | ForEach-Object { Join-Path $_ $Root }
$basePath = Split-Path -Parent $rootPath

if ((Test-Path -LiteralPath $rootPath) -and -not $Force) {
    throw "Test root already exists: $rootPath. Use -Force to recreate."
}

if (Test-Path -LiteralPath $rootPath) {
    Remove-Item -LiteralPath $rootPath -Recurse -Force
}

New-Item -ItemType Directory -Force -Path $basePath | Out-Null

$duplicateDirs = @(
    "dirs/a_keep_dir",
    "dirs/b_delete_dir",
    "dirs/c_delete_dir"
)

foreach ($dir in $duplicateDirs) {
    $full = Join-Path $rootPath $dir
    New-Item -ItemType Directory -Force -Path (Join-Path $full "src") | Out-Null
    New-Item -ItemType Directory -Force -Path (Join-Path $full "config") | Out-Null
    Set-Content -LiteralPath (Join-Path $full "src/main.txt") -Value "same directory payload" -Encoding UTF8
    Set-Content -LiteralPath (Join-Path $full "config/app.json") -Value '{ "name": "duplicate-dir" }' -Encoding UTF8
    Set-Content -LiteralPath (Join-Path $full "README.md") -Value "duplicate directory readme" -Encoding UTF8
}

New-Item -ItemType Directory -Force -Path (Join-Path $rootPath "dirs/unique_dir") | Out-Null
Set-Content -LiteralPath (Join-Path $rootPath "dirs/unique_dir/only.txt") -Value "unique directory" -Encoding UTF8

$duplicateFiles = @(
    "files/a_keep_file/report.txt",
    "files/b_delete_file/report-copy.txt",
    "files/c_delete_file/report-copy.txt"
)

foreach ($file in $duplicateFiles) {
    $full = Join-Path $rootPath $file
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $full) | Out-Null
    Set-Content -LiteralPath $full -Value "same standalone file payload" -Encoding UTF8
    Set-Content -LiteralPath (Join-Path (Split-Path -Parent $full) "marker.txt") -Value $file -Encoding UTF8
}

New-Item -ItemType Directory -Force -Path (Join-Path $rootPath "files/unique_file") | Out-Null
Set-Content -LiteralPath (Join-Path $rootPath "files/unique_file/unique.txt") -Value "unique file" -Encoding UTF8

Write-Host "Created: $rootPath"
