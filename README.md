# Boke

A clean, fast, keyboard-driven desktop RSS reader.

Built with [Tauri v2](https://tauri.app). Available for macOS, Windows, and Linux.

[![Latest Release](https://img.shields.io/github/v/release/dzania/Boke?style=flat-square&label=Download&color=blue)](https://github.com/dzania/Boke/releases/latest)
[![macOS](https://img.shields.io/badge/macOS-universal-black?style=flat-square&logo=apple)](https://github.com/dzania/Boke/releases/latest)
[![Windows](https://img.shields.io/badge/Windows-x64-0078D4?style=flat-square&logo=windows)](https://github.com/dzania/Boke/releases/latest)
[![Linux](https://img.shields.io/badge/Linux-x64-FCC624?style=flat-square&logo=linux&logoColor=black)](https://github.com/dzania/Boke/releases/latest)

![Boke screenshot](preview.png)

---

## What it does

Boke is an offline-first feed reader with a classic 3-panel layout: **feeds on the left, article list in the middle, reader on the right**. You paste a URL, it finds the RSS feed, and you start reading. No accounts, no cloud sync, no tracking.

### Features

- **Keyboard-first** -- vim-style navigation (`j`/`k`, `o`, `gg`, `G`), plus dozens of shortcuts
- **Smart filters** -- All Feeds, Unread, and Favourites with badge counts
- **Folders** -- drag and drop feeds into folders to organize your subscriptions
- **Dark mode** -- automatic (follows system) or manual toggle, persisted across sessions
- **System tray** -- minimize to tray, background refresh, OS notifications for new articles
- **Auto-updates** -- built-in updater checks GitHub Releases for new versions

### Supported formats

RSS 2.0, RSS 1.0 (RDF), and Atom.

---

## Keyboard shortcuts

### Navigation

| Key | Action |
|-----|--------|
| `j` / `Down` | Next article / scroll reader down |
| `k` / `Up` | Previous article / scroll reader up |
| `Enter` / `o` | Open selected article |
| `Escape` | Close reader or dialog |
| `g g` | Jump to top of list |
| `G` | Jump to bottom of list |
| `1`-`9` | Switch to nth feed |

### Reader

| Key | Action |
|-----|--------|
| `J` / `K` | Fast scroll in reader |
| `Space` | Page down |
| `Shift+Space` | Page up |

### Actions

| Key | Action |
|-----|--------|
| `m` | Toggle read / unread |
| `s` | Toggle favourite |
| `v` | Open in browser |
| `r` | Refresh current feed |
| `R` | Refresh all feeds |
| `Shift+A` | Mark all as read |
| `a` | Add feed |
| `b` | Toggle article list panel |
| `/` or `Cmd+K` | Search articles |
| `?` | Show keyboard shortcuts |
