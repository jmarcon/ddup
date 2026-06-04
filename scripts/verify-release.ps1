param(
    [switch]$SkipTagCheck
)

$ErrorActionPreference = "Stop"

if ($IsWindows -and -not $env:GITHUB_ACTIONS -and -not $env:CARGO_BUILD_TARGET) {
    $link = Get-Command link.exe -ErrorAction SilentlyContinue
    $gcc = Get-Command gcc -ErrorAction SilentlyContinue
    if ($link -and $gcc -and $link.Source -match "\\usr\\bin\\link\.exe$") {
        $gnuToolchain = "stable-x86_64-pc-windows-gnu"
        $toolchains = rustup toolchain list 2>$null
        if ($LASTEXITCODE -eq 0 -and ($toolchains -match $gnuToolchain)) {
            $env:RUSTUP_TOOLCHAIN = $gnuToolchain
        }
        $env:CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu"
        $env:CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = "gcc"
    }
}

function Invoke-Step {
    param(
        [string]$Name,
        [scriptblock]$Body
    )

    Write-Host "== $Name"
    & $Body
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

Invoke-Step "fmt" {
    cargo fmt --all --check
}

Invoke-Step "clippy" {
    cargo clippy --workspace --all-targets -- -D warnings
}

Invoke-Step "unit and integration tests" {
    cargo test --workspace
}

Invoke-Step "core has 75+ tests" {
    $output = cargo test -p ddup-core -- --list 2>&1
    if ($LASTEXITCODE -ne 0) {
        $output | Write-Host
        exit $LASTEXITCODE
    }
    $count = ($output | Select-String -Pattern ': test$').Count
    if ($count -lt 75) {
        throw "Expected at least 75 ddup-core tests, found $count"
    }
    Write-Host "Core tests: $count"
}

Invoke-Step "e2e tests" {
    ./scripts/run-e2e.ps1
}

Invoke-Step "CI and release workflows exist" {
    if (-not (Test-Path -LiteralPath ".github/workflows/ci.yml")) {
        throw "Missing .github/workflows/ci.yml"
    }
    if (-not (Test-Path -LiteralPath ".github/workflows/release.yml")) {
        throw "Missing .github/workflows/release.yml"
    }
}

Invoke-Step "docs" {
    cargo doc --no-deps --workspace
}

Invoke-Step "release build" {
    cargo build --release --workspace
}

if (-not $SkipTagCheck) {
    Invoke-Step "v0.1.0 tag points at HEAD" {
        $tags = git tag --points-at HEAD
        if ($tags -notcontains "v0.1.0") {
            throw "v0.1.0 does not point at HEAD"
        }
    }
}

Write-Host "Release verification passed."
