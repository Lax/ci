# ci — Personal Word Frequency Manager

A framework-agnostic Git-backed personal word frequency management tool. Automatically learns your input habits, syncs across devices via GitHub, and scans your blogs to build a personal vocabulary.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                          CLI (clap)                         │
│  init  import  export  sync  scan  status  device  daemon   │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                       Adapter Layer                         │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │ Rime (FFI)  │  │ Scan (jieba) │  │ Git (libgit2)    │   │
│  │ rime-api /  │  │ blog repos → │  │ fetch → merge →  │   │
│  │ rime_dict_  │  │ word freq    │  │ commit → push    │   │
│  │ manager     │  │ + bigrams    │  │                  │   │
│  └──────┬──────┘  └──────┬───────┘  └────────┬─────────┘   │
└─────────┼────────────────┼───────────────────┼──────────────┘
          │                │                   │
┌─────────▼────────────────▼───────────────────▼──────────────┐
│                       Core  Layer                            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌─────────────┐  │
│  │  Entry   │  │  Device  │  │  FreqDb  │  │   Merge     │  │
│  │ code,    │  │  id,     │  │ version, │  │ weighted by │  │
│  │ word,    │  │  name,   │  │ device,  │  │ device      │  │
│  │ freq,    │  │  total_  │  │ entries  │  │ entry count │  │
│  │ updated, │  │  entries │  │          │  │             │  │
│  │ prev,    │  └──────────┘  └──────────┘  └─────────────┘  │
│  │ source   │                                               │
│  └──────────┘                                               │
└─────────────────────────────────────────────────────────────┘
```

## Data Model

```rust
Entry       { code, word, freq, updated, prev: Option<String>, source: EntrySource }
EntrySource { Ime | Scan { repo, path } }
Device      { id, name, total_entries }
FreqDb      { version, device, entries }
DeviceRegistry → devices.yaml (known devices across syncs)
TriggerState → .trigger-state.yaml (12h cooldown)
```

**Merge strategy**: Weighted by each device's total entry count. A device with more
entries has more influence on the merged frequency.

## CLI

```
ci init [dir]            Initialize a freq-db repository
ci import                Import from Rime user dictionary
ci export                Export to Rime user dictionary
ci sync                  Sync with remote (fetch → merge → commit → push)
ci scan                  Scan blog repos for personal vocabulary
ci status                Show statistics
ci device list           List known devices
ci device add <name>     Register a device
ci daemon                Run one sync cycle (respects cooldown)
ci daemon --watch        Watch Rime directory for changes, auto-sync
```

## Quick Start

```bash
# 1. Initialize a repo
ci init ~/my-freq-db
cd ~/my-freq-db

# 2. Import from Rime (reads double_pinyin_abc user dictionary)
ci import

# 3. Add a remote (GitHub)
git remote add origin git@github.com:user/ci-wordfreq.git
ci sync

# 4. (optional) Configure blog repos to scan
#     edit ci.yaml → add scan_repos:
#     scan_repos:
#       - ~/projects/lax.github.com
#       - ~/projects/yuedulijie.com
ci scan

# 5. Export back to Rime
ci export
```

## Daemon Mode

```bash
# Run once (for systemd timer)
ci daemon

# Continuous watch mode (for systemd service)
ci daemon --watch
```

The daemon respects a 12-hour cooldown window. In `--watch` mode it monitors
the Rime user directory for changes and triggers a full cycle (import → sync →
export) when activity is detected and the cooldown has elapsed.

### systemd user service

```bash
cp contrib/ci-daemon.service ~/.config/systemd/user/
systemctl --user enable ci-daemon
systemctl --user start ci-daemon
```

## Dependencies

| Component          | Crate              | Notes                        |
|--------------------|--------------------|------------------------------|
| CLI                | clap               | derive macros                |
| Serialization      | serde + serde_yaml | YAML primary, JSON secondary |
| Git                | git2               | libgit2 bindings             |
| IME integration    | rime-api / librime-sys | FFI, falls back to rime_dict_manager |
| Chinese tokenizer  | jieba-rs           | blog scanning                |
| File watching      | inotify            | daemon --watch mode          |
| UUID               | uuid               | device identification        |

## Storage Layout

```
~/.local/share/fcitx5/rime/
├── sync/                    # Rime sync directory
│   ├── {installation_id}/
│   │   └── *.userdb.txt
│   └── ci_export_*/         # Temporary export dir (auto-cleaned)
├── ci_export.userdb.txt     # Temporary (auto-cleaned)
└── ...

~/my-freq-db/
├── ci.yaml                  # Configuration
├── entries.yaml             # Word frequency database
├── devices.yaml             # Known devices registry
├── .trigger-state.yaml      # Cooldown state (daemon mode)
├── split/                   # Sharded entries (future)
├── .gitignore
└── ...
```

## Implementation Status

- P0 ✓ Core model + store + merge + rime adapter + init/import/export
- P1 ✓ Git sync + device management + status CLI
- P2 ✓ Blog scanning with jieba-rs tokenization
- P3 ✓ Daemon with 12h cooldown + systemd service
- P4 ✓ librime FFI replacing rime_dict_manager process calls
