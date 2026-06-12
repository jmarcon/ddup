# ddup

CLI/TUI Rust para encontrar diretórios e arquivos duplicados, comparar múltiplas roots e aplicar ações de limpeza com rastreio em SQLite.

![ddup TUI](docs/assets/tui-screenshot.svg)

## Instalação

### Plataformas Suportadas

- **Windows**: x64 (.zip)
- **Linux**: x64 (.tar.gz)
- **macOS**: x64 (.tar.gz)

### Downloads

Os binários compilados estão disponíveis na página de [Releases](https://github.com/jmarcon/ddup/releases).

- **Releases Oficiais**: Versões estáveis (ex: `v0.1.0`).
- **Latest Build**: Build automático da branch `main` com as últimas alterações (pre-release).

### Instalação Manual

Pré-requisitos:

- Rust stable.
- PowerShell 7 para scripts E2E.
- Nerd Fonts opcional, para ícones na TUI.

Instalação local via Cargo:

```powershell
cargo install --path crates/ddup
```

Build manual:

```powershell
cargo build --release -p ddup
```

## Uso Rápido

```powershell
ddup C:\path\to\scan
ddup C:\path\to\scan --mode flat --rescan
ddup C:\left C:\right --mode smart
ddup C:\path\to\scan --db C:\tmp\ddup.sqlite --no-tui
ddup C:\path\to\scan --memory-db
ddup C:\path\to\scan --no-icons
```

Com um path, `ddup` procura duplicatas dentro da root.

Com dois ou mais paths, `ddup` compara duplicatas entre roots e ignora duplicatas locais de uma única root.

## Opções CLI

| Opção | Uso |
|---|---|
| `paths...` | Uma ou mais roots para scan |
| `--mode smart` | Modo padrão; suprime arquivos dentro de diretórios duplicados |
| `--mode flat` | Lista todos os arquivos duplicados |
| `--rescan` | Ignora resultado salvo e executa novo scan |
| `--no-walk` | Abre TUI sem executar scan |
| `--exit-after-scan` | Fecha TUI após scan concluir |
| `--no-tui` | Executa scan e imprime resumo no terminal |
| `--db <path>` | Usa arquivo SQLite específico |
| `--memory-db` | Usa SQLite em memória |
| `--in-memory-db` | Alias de `--memory-db` |
| `--no-icons` | Desativa ícones Nerd Fonts |
| `--help` | Mostra ajuda da CLI |

`--memory-db` conflita com `--db` e não cria arquivo.

## SQLite

Sem `--db`, o SQLite fica em:

```text
Windows: %LOCALAPPDATA%\ddup\scans.db
```

Para testes isolados:

```powershell
ddup C:\repo --db C:\tmp\ddup.sqlite --no-tui
```

Para execução temporária:

```powershell
ddup C:\repo --memory-db
```

A TUI mostra o caminho do banco no topo.

Mais detalhes: [docs/sqlite.md](docs/sqlite.md).

## Modos

Smart é o default.

Ele remove da visualização de arquivos os arquivos duplicados que já pertencem a uma pasta duplicada.

Flat mostra todos os arquivos duplicados.

Use Flat quando cada arquivo duplicado importa, mesmo dentro de diretórios duplicados.

## Ordenação

A ordenação padrão é `TotalSize Desc`.

Para diretórios duplicados, isso ordena pelo tamanho de cada cópia duplicada.

Use:

- `s` para alternar campo de sort.
- `r` para alternar ordem.

Campos disponíveis:

- `TotalSize`
- `FileCount`
- `Path`

## TUI

| Tecla | Ação |
|---|---|
| `j/k` ou setas | Navegar |
| `PgUp/PgDown` | Navegar por página |
| mouse | Selecionar e rolar |
| `Enter` | Abrir/fechar grupo |
| `h/l` | Fechar/abrir grupo |
| `f` | Alternar Dirs / Files Smart / Files Flat |
| `s` | Alternar campo de sort |
| `r` | Alternar ordem do sort |
| `Delete` | Deletar item selecionado com confirmação |
| `m` | Mover item selecionado |
| `o` | Abrir no file manager |
| `c` | Copiar caminho completo |
| `i` | Ativar/desativar ícones Nerd Fonts |
| `Space`, `d` ou `x` | Marcar/desmarcar para delete |
| `K` | Marcar/desmarcar para manter |
| `u` ou `Backspace` | Limpar marca |
| `a` | Aplicar marcas |
| `F5` ou `Ctrl+R` | Re-scan |
| `?` | Help |
| `q` ou `Esc` | Sair |
| `Ctrl+C` | Sair |

Quando o cursor está em `Dir group` ou `File group`, marcações são aplicadas a todos os itens do grupo.

Após aplicar marcas, a TUI recarrega o scan atual para mostrar status atualizado.

## Marcação e Limpeza

Fluxo recomendado:

1. Abra um grupo.
2. Marque cópias para exclusão com `d`, `x` ou `Space`.
3. Marque itens a preservar com `K`, se necessário.
4. Revise o painel `Details`.
5. Aplique com `a`.

Estados visuais:

- `[ ]` sem decisão.
- `[x]` marcado para delete.
- `[K]` marcado para keep.
- `Deleted`, `Moved`, `Kept` após ação aplicada.

## Hashes

Cada arquivo recebe hash SHA-256.

Cada diretório recebe hash derivado de:

- hashes dos arquivos filhos diretos.
- hashes das pastas filhas diretas.

Isso torna a comparação determinística e independente da ordem de leitura do filesystem.

## API Rust

```rust
use ddup_core::{scan, scan_roots, ScanMode, WalkConfig};

let result = scan(".".as_ref(), &WalkConfig::default(), ScanMode::Smart, None)?;
let compared = scan_roots(
    &["C:/left".into(), "C:/right".into()],
    &WalkConfig::default(),
    ScanMode::Smart,
    None,
)?;
println!("{}", result.summary.wasted_bytes);
println!("{}", compared.summary.wasted_bytes);
# Ok::<(), ddup_core::CoreError>(())
```

## GUI Integration

O core persiste `tree_nodes` em SQLite com dados prontos para GUI.

Use:

- `Db::fetch_children` para lazy loading.
- `Db::fetch_subtree` para treemap/sunburst.
- `Db::fetch_dir_groups` para diretórios duplicados.
- `Db::fetch_file_groups` para arquivos duplicados.

Campos pré-computados:

- `size_recursive`
- `dup_status`
- `extension`
- `wasted_bytes`
- `content_hash`

O core não define cores nem layout.

## Scripts

| Script | Uso |
|---|---|
| `scripts/new-test-fixtures.ps1` | Cria fixtures |
| `scripts/remove-duplicate-dirs.ps1` | Remove diretórios duplicados |
| `scripts/validate-duplicate-dirs-removed.ps1` | Valida limpeza de diretórios |
| `scripts/remove-duplicate-files.ps1` | Remove arquivos duplicados |
| `scripts/validate-duplicate-files-removed.ps1` | Valida limpeza de arquivos |
| `scripts/clean-test-fixtures.ps1` | Remove fixtures |
| `scripts/run-e2e.ps1` | Executa fluxo E2E |
| `scripts/verify-release.ps1` | Executa validação completa |

## Testes

A integridade do projeto é garantida por testes unitários, de integração e fluxos E2E.

### Execução Local

```powershell
# Roda todos os testes da workspace
cargo test --workspace

# Validação completa (lint, tests, e2e, doc, build)
./scripts/verify-release.ps1
```

### CI/CD

Todos os Pull Requests e pushes para `main` passam por validação automática no GitHub Actions em:
- Ubuntu (Linux)
- macOS
- Windows

## Release

O processo de release é automatizado:

1.  **Push para `main`**: Gera automaticamente um **Latest Build** (pre-release) com binários atualizados.
2.  **Tag `v*`**: Gera uma **Release Oficial** estável.

Mais detalhes técnicos: [docs/release.md](docs/release.md).
