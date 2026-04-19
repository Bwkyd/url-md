# url-md

[![Release](https://img.shields.io/github/v/release/Bwkyd/url-md)](https://github.com/Bwkyd/url-md/releases)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](./LICENSE)
[![Stars](https://img.shields.io/github/stars/Bwkyd/url-md?style=social)](https://github.com/Bwkyd/url-md)

English · [中文 →](./README.md)

**Any URL → clean Markdown.** MCP-native · Rust single binary · Apache-2.0.

## Acknowledgments & Recommendation

The product idea behind this project is inspired by **[42md.cc](https://42md.cc)** — a commercial "AI knowledge compiler" SaaS by [Huoshui AI Lab](https://42md.cc) that compiles webpages, PDFs, and audio into structured Markdown knowledge cards and topic maps.

**Prefer 42md when:**

- You need AI-powered knowledge card extraction, topic maps, and cross-format compilation → **[visit 42md.cc](https://42md.cc)** and subscribe to support the creators
- It's a full knowledge-workflow tool, not just a format converter

## Install

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/Bwkyd/url-md/main/install.sh | bash
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/Bwkyd/url-md/main/install.ps1 | iex
```

Installs to `~/.url-md/bin/url-md` (Windows: `%USERPROFILE%\.url-md\bin\url-md.exe`). The script tells you how to add it to `PATH`.

<details>
<summary>Alternatives</summary>

**Rust users** — one-liner from git:
```bash
cargo install --git https://github.com/Bwkyd/url-md url-md --locked
```

**From source** — if you don't want a global install:
```bash
git clone https://github.com/Bwkyd/url-md.git
cd url-md && cargo build --release
./target/release/url-md <URL>
```

**Pin a version** — the installer accepts a tag:
```bash
curl -fsSL https://raw.githubusercontent.com/Bwkyd/url-md/main/install.sh | bash -s v0.1.2
```
</details>

## Usage

```bash
url-md <URL>              # Print Markdown to stdout (no images saved)
url-md <URL> -o out/      # Save to folder + auto-download images to out/assets/
```

Other flags: `url-md --help` · `--no-assets` skip images · `--assets <DIR>` custom image dir · `--verbose / --quiet` · `--timeout`.

**Exit codes**: 0=ok · 10=network · 11=anti-bot · 12=paywall · 13=auth-required · 20=parse · 30=I/O · 99=internal

## Use in Claude Code / Cursor (MCP)

url-md ships as a native MCP server — AI agents can call it directly, no shell wrapping needed.

### Claude Code

```bash
claude mcp add url-md -- url-md serve --mcp
```

### Cursor / Cline

`mcp.json`:
```json
{
  "mcpServers": {
    "url-md": {
      "command": "url-md",
      "args": ["serve", "--mcp"]
    }
  }
}
```

Once configured, the agent can call one tool:
- **`md(url, timeout_seconds?)`** — returns Markdown (byte-identical to `url-md md <url>` CLI output)

## What it looks like

```bash
url-md https://mp.weixin.qq.com/s/AMJBh90iNEZBRLY3iWsYxQ -o out/
```

**File 1**: `out/2026-04-17-mp_weixin_qq_com-bestseller.md`

```markdown
---
title: How Bestsellers Waste Your Time
author: Niklas Göke
publish_time: 2026-04-17 07:42
cover_url: https://mmbiz.qpic.cn/.../0?wx_fmt=jpeg
extract_method: weixin
word_count: 3247
reading_time_minutes: 11
source_url: https://mp.weixin.qq.com/s/AMJBh90iNEZBRLY3iWsYxQ
source_adapter: weixin
fetched_at: 2026-04-17T16:17:48Z
---

**Open Mind Insights**

![img](assets/img-0001.gif)

Bestsellers are popular, but are they worth reading? The author dissects one…
```

**File 2**: `out/assets/img-0001.gif` … `img-0008.png` (all 8 images downloaded, Markdown references rewritten to relative paths — **works offline**).

## What it can grab

| Site | Support |
|---|---|
| WeChat official account permalinks (`mp.weixin.qq.com/s/*`) | ✅ Complete (images / author / publish time / cover) |
| Hacker News / Rust Book / static blogs | ✅ generic fallback |
| Multi-article list homepages | ✅ all `<article>` tags merged |


## Status

**v0.1.x · single-URL only**. Batch / HTTP / MCP / auth walls are on the roadmap.

## Contributing

New site adapter: `src/url-md-adapters/src/<site>.rs` (look at the existing `weixin.rs`). PRs must include `Signed-off-by:` (DCO).

## License

Apache-2.0 — see [LICENSE](./LICENSE).
