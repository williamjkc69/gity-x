<div align="center">
  <img src="assets/icon.png" alt="gity x logo" width="96" />
  <h1>gity x</h1>
  <p><strong>Fast, lightweight, cross-platform Git GUI built in Rust</strong></p>
  <p>Inspired by GitKraken, VS Code Git Graph, and Antigravity's minimalist dark UI.</p>
  <p>
    <img src="https://img.shields.io/badge/Rust-2021-orange?style=flat-square&logo=rust" alt="Rust" />
    <img src="https://img.shields.io/badge/Platform-macOS%20%7C%20Windows%20%7C%20Linux-8BA4F8?style=flat-square" alt="Platforms" />
    <img src="https://img.shields.io/badge/License-MIT-52C482?style=flat-square" alt="MIT License" />
  </p>
</div>

---

## Features

- **Visual Git Commit Graph**: Color-coded lanes, Rounded (Bézier) & Angular edge styles, `HEAD` ring, distinct merge nodes, `WIP` row, and branch/remote/tag badges.
- **Per-Column Contextual Tooltips**: Hover over Graph, Commit/Refs, Author (with initials avatar, commit stats, committer, co-authors & GPG/SSH signature status), Date (human-formatted), or Hash (full SHA-1).
- **Agent Mode (Maximized Working Changes)**: Dedicated full-width review for Unstaged, Untracked, Staged files, and Stashes with **Split (Side-by-Side)** and **Unified** diffs.
- **Chrome-Style Multi-Repo Tabs**: Open multiple repositories simultaneously with `O(1)` memory footprint (inactive tabs store metadata only) and 0% idle CPU.
- **Complete Git Operations**:
  - Cherry-pick, Revert, Checkout, Merge (normal & `--no-ff`), Rebase, Reset (`--soft` / `--mixed` / `--hard`)
  - Conflict banner with Continue / Abort for Merge, Rebase, and Cherry-pick
  - Searchable branch switcher dropdown + Create Branch From (HEAD, local, remote, tag, or commit) + Rename / Force Delete (`-D`)
  - Compare any 2 commits or compare commit vs Working Tree (`vs WIP`)
  - Changed files in Tree or Flat list view with `+add -del` stats
  - Tags (create lightweight/annotated, push, delete) and Stashes (save with message, preview diff, apply, pop, drop)
  - Lightweight Fetch / Pull / Push indicators (`git ls-remote` one-shot check, zero polling)
- **Polished UX**:
  - Embedded **Plus Jakarta Sans** & **JetBrains Mono** typography
  - Collapsible left & right sidebars with custom deterministic resize splitter
  - Humanized Error Modal with full raw output & 1-click Copy for debugging
  - Stacked bottom-right floating success toasts (5s auto-dismiss)
  - Native OS menu bar + English (default) & Español i18n persisted in `~/.config/gity-x/config.txt`

## Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| `Cmd/Ctrl + O` or `Cmd/Ctrl + N` | Open repository in new tab |
| `Cmd/Ctrl + W` | Close active tab |
| `Cmd/Ctrl + 1..9` / `Ctrl + Tab` | Switch repository tabs |
| `Cmd/Ctrl + E` | Toggle **Git Graph** / **Agent Mode** |
| `Cmd/Ctrl + R` | Refresh repository & check remote updates |
| `Cmd/Ctrl + F` | Focus search / filter |
| `Cmd/Ctrl + B` | Toggle left sidebar |
| `Cmd/Ctrl + I` | Toggle right inspector |
| `Cmd/Ctrl + ,` | Open Settings |
| `Cmd/Ctrl + /` | Open Keyboard Shortcuts |
| `Esc` | Close active modal |

## Build & Run from Source

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- `git` available in `PATH`

### Run

```bash
git clone https://github.com/williamjkc69/gity-x.git
cd gity-x
cargo run --release
```

You can also pass a repository path directly:

```bash
cargo run --release -- /path/to/your/repo
```

## Cross-Platform Packaging Roadmap

Planned automated GitHub Releases builds for:
- **macOS**: Universal `.dmg` app bundle
- **Windows**: `.exe` / `.msi` installer
- **Ubuntu / Linux**: `.deb` & `.AppImage`

## Author

Created by **[@williamjkc69](https://github.com/williamjkc69)**.

## License

MIT — see [LICENSE](LICENSE).
