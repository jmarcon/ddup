# SQLite

`ddup` usa SQLite para armazenar resultados do scan, status de ações e dados de árvore.

## Modos de Banco

Arquivo padrão:

```powershell
ddup C:\repo
```

Arquivo explícito:

```powershell
ddup C:\repo --db C:\tmp\ddup.sqlite
```

In-memory:

```powershell
ddup C:\repo --memory-db
```

Alias:

```powershell
ddup C:\repo --in-memory-db
```

`--memory-db` não cria arquivo e conflita com `--db`.

## Caminho Padrão

No Windows:

```text
%LOCALAPPDATA%\ddup\scans.db
```

Para testes, `DDUP_DATA_LOCAL_DIR` muda a base do caminho padrão.

```powershell
$env:DDUP_DATA_LOCAL_DIR="C:\tmp\ddup-data"
ddup C:\repo --no-tui
```

## Persistência

Ao final do scan, são salvos:

- resumo do scan.
- grupos de diretórios duplicados.
- grupos de arquivos duplicados.
- árvore de diretórios e arquivos.
- hashes de conteúdo.
- status de ações.

Na TUI, ações como delete, move e keep atualizam o SQLite.

Depois de aplicar marcas, a TUI recarrega os dados para exibir o estado atual.

## Tabelas Principais

| Tabela | Conteúdo |
|---|---|
| `scans` | Root, modo, data e resumo |
| `dup_groups` | Grupos de diretórios duplicados |
| `dup_entries` | Cópias de cada diretório duplicado |
| `dup_file_groups` | Grupos de arquivos duplicados |
| `dup_file_entries` | Cópias de cada arquivo duplicado |
| `tree_nodes` | Árvore completa com estatísticas |

## Dados para GUI

`tree_nodes` já inclui dados prontos para visualização:

- `path`
- `parent_path`
- `kind`
- `size_bytes`
- `size_recursive`
- `file_count`
- `dup_status`
- `wasted_bytes`
- `extension`
- `content_hash`
- `dir_group_id`
- `file_group_id`

Use `fetch_children` para navegação incremental.

Use `fetch_subtree` para visualizações agregadas.
