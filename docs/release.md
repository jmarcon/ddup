# Release

Prerequisites:

- Git remote configured.
- GitHub authentication available for `git push` and Actions.
- Clean working tree.
- `v0.1.0` tag pointing to the release commit.

Commands:

```powershell
./scripts/verify-release.ps1
git push origin main
git push --force origin v0.1.0
```

Expected result:

- CI workflow green on Ubuntu, macOS, and Windows.
- Release workflow creates GitHub Release assets:
  - `ddup-linux-x64.tar.gz`
  - `ddup-macos-x64.tar.gz`
  - `ddup-windows-x64.zip`

## Local Verification

`verify-release.ps1` runs:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- core test count validation.
- `scripts/run-e2e.ps1`
- workflow file validation.
- `cargo doc --no-deps --workspace`
- `cargo build --release --workspace`
- tag check.

Use `-SkipTagCheck` before moving the tag:

```powershell
./scripts/verify-release.ps1 -SkipTagCheck
```

## GitHub Checks

After pushing:

```powershell
gh run list --repo jmarcon/ddup --limit 10
gh run watch <run-id> --repo jmarcon/ddup --exit-status
```

## Release Assets

The release workflow builds:

- Linux x64 tarball.
- macOS x64 tarball.
- Windows x64 zip.

Each artifact contains the `ddup` binary for the target OS.
