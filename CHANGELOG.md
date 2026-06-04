# Changelog

## [Unreleased]

### Added

- Documentação expandida no `README.md`.
- Guia de uso em `docs/usage.md`.
- Guia SQLite em `docs/sqlite.md`.
- SQLite in-memory com `--memory-db` e `--in-memory-db`.

### Testing

- E2E para SQLite in-memory.

## [0.1.0] - 2026-06-04

### Added

- Workspace Rust com `ddup-core` e `ddup`.
- CLI/TUI com nome `ddup`.
- Scan de um ou mais paths.
- Comparação entre múltiplas roots.
- Modo `Smart`.
- Modo `Flat`.
- SQLite padrão em `%LOCALAPPDATA%\ddup\scans.db`.
- SQLite customizado com `--db`.
- TUI com tema Dracula.
- TUI aberta imediatamente durante scan.
- Feedback visual de scan com steps, spinner e progresso.
- Painel inferior para erros de scan.
- Navegação por teclado, mouse, scroll, `PgUp` e `PgDown`.
- Re-scan pela TUI com `F5` e `Ctrl+R`.
- Saída com `q`, `Esc` e `Ctrl+C`.
- Help modal estruturado.
- Cópia do caminho completo com `c`.
- Ícones Nerd Fonts para arquivos, pastas e grupos.
- Flag `--no-icons`.
- Grupos expansíveis e recolhíveis.
- Caminhos relativos ao root na lista.
- Painel de detalhes com status, tamanho, hash, root e cópias.
- Ordenação por tamanho, repetição e path.
- Ordenação padrão por maior tamanho duplicado.
- Marcação de delete, keep e clear.
- Marcação aplicada ao grupo inteiro quando cursor está no grupo.
- Aplicação de marcações com refresh de status.
- Ações de delete, move e open.
- Hash SHA-256 por arquivo.
- Hash de diretório baseado nos hashes dos filhos diretos.
- Persistência de `content_hash` em `tree_nodes`.
- Scripts E2E para criação, remoção, validação e limpeza de fixtures.
- Workflow CI em Ubuntu, macOS e Windows.
- Workflow Release com artefatos Linux, macOS e Windows.

### Changed

- Persistência do scan integrada ao fluxo da TUI.
- Sort `TotalSize Desc` corrigido para ordenar pelo tamanho real do grupo.
- Help e atalhos revisados para termos mais claros.
- Toolchain CI ajustada para `stable` portátil.
- Line endings fixados em LF para CI Windows.

### Fixed

- TUI não aparecendo ao executar `ddup`.
- Logs de erro aparecendo fora do painel de erros.
- Progresso sem porcentagem e com texto sobreposto.
- Scan travado visualmente após 100%.
- Seleção invisível na lista.
- Detalhes inválidos ao selecionar item.
- Navegação pulando item.
- `Esc` fechando a TUI ao invés de fechar apenas o help.
- Diretórios não indentados.
- Impossibilidade de fechar grupos.
- SQLite padrão não persistindo corretamente.
- Filtro de subtree incluindo paths irmãos com prefixo parecido.
- CI falhando por toolchain Windows específica.
- CI Windows falhando por newline.

### Testing

- 90+ testes unitários e integração.
- E2E para persistência padrão.
- E2E para SQLite customizado.
- E2E para comparação multi-root.
- E2E para remoção de diretórios duplicados.
- E2E para remoção de arquivos duplicados.
- `scripts/verify-release.ps1` valida fmt, clippy, testes, E2E, docs, build release e tag.
