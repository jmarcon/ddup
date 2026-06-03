param(
    [switch]$SkipTagCheck
)

$ErrorActionPreference = "Stop"

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

Invoke-Step "e2e tests" {
    ./scripts/run-e2e.ps1
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
