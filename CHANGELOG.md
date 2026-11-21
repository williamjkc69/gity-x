# Changelog

All notable changes to **gity x** are documented in this file.

## [v0.4.0] — AI Pull Request Code Reviewer & Detailed Release Notes

### Added
- **Opt-In AI Pull Request Code Review (`Settings → AI Reviewer`)**: Disabled by default; enable in Settings and configure **OpenAI**, **Anthropic (Claude)**, **OpenRouter**, **Google Gemini (native REST API)**, or **Custom** OpenAI-compatible endpoints with community-recommended code review model presets (with badges & tooltips), 1-click known Base URL presets (**Ollama**, **LM Studio**, **vLLM**, **DeepSeek**, **Groq**, **xAI**, **Mistral**, **Together**, **Fireworks**, **LiteLLM**) or custom URLs, masked API key (`0600`), live `Test` connection, and custom team rules.
- **Comprehensive 6-Dimension Audit Report**: Evaluates PR commits and unified diff patches (filtering out lockfiles/generated noise) for **Verdict & Quality Score**, **Security & Cybersecurity (OWASP)**, **Bugs & Edge Cases**, **Performance & Benchmarks**, **Code Quality / Linting**, and **Prioritized Action Items**.
- **One-Click PR Sharing Actions**: **Post as PR Comment** (mentioning `@author`), **Request Changes** with the report, **Edit in Composer** before posting, or **Copy Report** as Markdown.
- **Dedicated AI Error Diagnostics & Help Guide**: AI errors show clear AI-specific HTTP 401/404/429 diagnostics (`AI Reviewer Error` modal + direct `Configure AI` button) and EN/ES Help Center troubleshooting.
- **Hardened HTML & Markdown Renderer (`src/ui/markdown.rs`)**: Renders embedded HTML (`<details>/<summary>` including nested drawers, `<table>`, `<h1>`–`<h6>`, `<b>`/`<i>`/`<s>`/`<code>`/`<kbd>`, `<pre><code class="language-...">`, `<blockquote>`, `<ol>`/`<ul>`/`<li>`, `<input type="checkbox">`, `<a href>`, `<img src alt>`, `<br>`/`<hr>`, named/numeric/hex entities) seamlessly alongside full GitHub Flavored Markdown (GitHub Alerts `[!NOTE]`/`[!TIP]`/`[!IMPORTANT]`/`[!WARNING]`/`[!CAUTION]`, nested inline styles, reference links, autolinks, bare URLs, Setext headings, Dependabot nested badges, balanced-paren URLs, escaped table pipes).
- **Automated Detailed Release Notes**: GitHub Actions release pipeline now extracts structured Added / Fixed / Changed release notes from `CHANGELOG.md` for every release.

### Fixed
- **Google Gemini Endpoint & OpenAI Reasoning Models**: Switched Gemini to native `v1beta/models/{model}:generateContent` with `x-goog-api-key` (fixes HTTP 404 when pasting `/v1beta`, omits custom temperature on Gemini 3/4), and added `max_completion_tokens` + `developer` role support for OpenAI `o1`/`o3`/`o4`/`gpt-5`/`gpt-6` models.
- **HTML & Markdown Edge-Case Bugs**: Fixed UTF-8 byte-offset panics on multi-byte uppercase characters (`İ`), nested `<details>`/`<summary>` scoping, generic type preservation (`Result<User>`, `Option<T>`), inline/fenced code preservation for HTML/comments/entities, multi-line `<li>`/`<h1>`/`<blockquote>` formatting, attribute boundary collisions (`data-src` vs `src`), `C#` headings, `snake_case` underscore guards, and `<script>`/`<style>` stripping.

## [v0.3.2] — Collapsible PRs, Markdown Rendering, Help Center & macOS Paste Fix

### Added
- **Collapsible Pull Requests List**: Full-width accordion PR list (`▸`/`▾`) that lazily loads full details only when clicked.
- **Automatic Markdown Rendering**: Renders GitHub/GitLab/Bitbucket PR descriptions and comments with headings, fenced code blocks, tables, task checklists (`✓`/`□`), blockquotes, inline code, and clickable links (strips HTML template comments).
- **Redesigned Tabbed Settings**: Separate `General & Graph` and `Git Providers & Tokens` tabs with descriptions, Show/Hide toggle, live `Test` token verification, and step-by-step guides + official links.
- **Help & Troubleshooting Center (`?`)**: Built-in searchable troubleshooting center (EN/ES) with copyable commands and direct fix actions.

### Fixed
- **macOS `Cmd+V` / `Cmd+C` / `Cmd+X` / `Cmd+A` Paste Fix**: Removed macOS `PredefinedMenuItem` Edit menu that intercepted clipboard shortcuts before `egui` received them.
- **GitHub HTTP 422 Error Details**: Fixed `extract_api_error_msg` to parse plain string items in GitHub's `"errors": [...]` array (e.g. `"Can not approve your own pull request"`).
- **Bitbucket Repo/Workspace Access Tokens**: Automatically uses `Bearer` auth for `ATCTT3x...` tokens even if `User / Email` is filled; sanitizes pasted `Bearer`/`PRIVATE-TOKEN` prefixes.
- **Cross-Platform Polish**: OS-aware `Cmd`/`Ctrl` labels, Windows `%APPDATA%` config display & `USERPROFILE` folder dialog, Linux `.deb` auto-updater guard.

## [v0.3.1] — macOS `.app` GitHub CLI Discovery Fix

### Fixed
- **GitHub Private Repos HTTP 404 in macOS `.app` Bundles**: Discovers `gh` CLI across `/opt/homebrew/bin`, `/usr/local/bin`, `/opt/local/bin`, `~/.local/bin`, and Windows standard install directories even when launched from Finder/Dock with a minimal `PATH`.

## [v0.3.0] — Unified Pull Requests & Auto-Updater

### Added
- **Unified Pull Requests / Merge Requests (`Cmd/Ctrl+P`)**: Support for **GitHub**, **GitLab**, and **Bitbucket Cloud** behind one unified UI with Conversation, Commits, Files changed diffs, Approve, Request Changes, Comment, Close/Decline, Reopen, Checkout branch, and Merge strategies (`Merge`, `Squash`, `Rebase`).
- **SHA-256 Verified Auto-Updater**: Checks GitHub Releases once on startup, verifies `SHA256SUMS.txt` with pure-Rust SHA-256, replaces macOS `.app` / Windows `.exe` / Linux `.AppImage` atomically, and offers 1-click restart.

## [v0.2.2] — Keychain Popup Prevention & CI Fixes

### Fixed
- Made automatic remote update checks (`git ls-remote`) opt-in in Settings to prevent repeated macOS Keychain password prompts on HTTPS repos.
- Added comprehensive OS installation & security verification documentation (`README.md`, `SECURITY.md`, `scripts/install-macos.sh`).
