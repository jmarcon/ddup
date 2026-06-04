# Uso

Este documento detalha os fluxos principais do `ddup`.

## Scan Simples

```powershell
ddup C:\repo
```

Abre a TUI, inicia scan e persiste no SQLite padrão.

## Scan Sem TUI

```powershell
ddup C:\repo --no-tui
```

Imprime:

- caminho do SQLite.
- total de diretórios.
- total de arquivos.
- bytes desperdiçados.

## Comparar Duas Pastas

```powershell
ddup C:\left C:\right
```

Duplicatas só entram no resultado se existirem em mais de uma root.

Duplicatas internas apenas em `C:\left` ou apenas em `C:\right` são ignoradas.

## Múltiplas Roots

```powershell
ddup C:\repo-a C:\repo-b C:\repo-c --mode smart
```

Funciona como comparação entre roots.

## Smart vs Flat

Smart:

```powershell
ddup C:\repo --mode smart
```

Flat:

```powershell
ddup C:\repo --mode flat
```

Use Smart para foco em diretórios duplicados.

Use Flat para auditar todos os arquivos duplicados.

## Re-scan

```powershell
ddup C:\repo --rescan
```

Na TUI:

```text
F5
Ctrl+R
```

## Banco Temporário

```powershell
ddup C:\repo --memory-db
```

Útil para auditoria rápida sem persistência em disco.

## Banco Específico

```powershell
ddup C:\repo --db C:\tmp\ddup.sqlite
```

Útil para fixtures, automações e inspeção manual.

## Aplicar Ações

Na TUI:

1. Navegue até grupo ou item.
2. Use `d`, `x` ou `Space` para marcar delete.
3. Use `K` para marcar keep.
4. Use `a` para aplicar.

Em cima de `Dir group` ou `File group`, a marca vale para todos os itens do grupo.

Após aplicar, a lista é recarregada com status atualizado.
