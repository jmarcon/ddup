# Release v0.1.0

Prerequisites:

- Git remote configured.
- GitHub authentication available for `git push` and Actions.

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
