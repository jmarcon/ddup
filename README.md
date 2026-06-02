# ddup

CLI/TUI Rust para detectar diretórios e arquivos duplicados.

## Uso

```powershell
cargo install --path crates/ddup-tui
ddup-tui C:\path\to\scan --mode smart
```

## API Rust

```rust
use ddup_core::{scan, ScanMode, WalkConfig};

let result = scan(".".as_ref(), &WalkConfig::default(), ScanMode::Smart, None)?;
println!("{}", result.summary.wasted_bytes);
# Ok::<(), ddup_core::CoreError>(())
```

## GUI Integration

O core persiste `tree_nodes` em SQLite v2.

Use `Db::fetch_children` para lazy loading de treemap.

Use `Db::fetch_subtree` para sunburst.

`size_recursive`, `dup_status`, `extension` e `wasted_bytes` já vêm pré-computados.

## Smart Mode

Smart suprime arquivos duplicados dentro de diretórios duplicados.

Flat mostra todos os arquivos duplicados.

