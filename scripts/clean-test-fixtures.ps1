param(
    [string]$Base = "work/ddup-e2e"
)

$ErrorActionPreference = "Stop"
$basePath = Join-Path (Resolve-Path -LiteralPath ".") $Base

if (Test-Path -LiteralPath $basePath) {
    Remove-Item -LiteralPath $basePath -Recurse -Force
}

Write-Host "Removed: $basePath"
