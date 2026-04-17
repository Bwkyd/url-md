# url-md

[![Release](https://img.shields.io/github/v/release/Bwkyd/url-md)](https://github.com/Bwkyd/url-md/releases)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](./LICENSE)
[![Stars](https://img.shields.io/github/stars/Bwkyd/url-md?style=social)](https://github.com/Bwkyd/url-md)

[English →](./README.en.md) · 中文

**任意 URL → 干净 Markdown**。专注微信公众号 + MCP 协议 · Rust 单二进制 · Apache-2.0。

## 安装

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/Bwkyd/url-md/main/install.sh | bash
```

### Windows(PowerShell)

```powershell
irm https://raw.githubusercontent.com/Bwkyd/url-md/main/install.ps1 | iex
```

装到 `~/.url-md/bin/url-md`(Windows 为 `%USERPROFILE%\.url-md\bin\url-md.exe`)。脚本会提示如何加 PATH。

<details>
<summary>其他方式</summary>

**Rust 用户** — 一行从 git 装:
```bash
cargo install --git https://github.com/Bwkyd/url-md url-md --locked
```

**从源码构建** — 不想全局安装:
```bash
git clone https://github.com/Bwkyd/url-md.git
cd url-md && cargo build --release
./target/release/url-md <URL>
```

**指定版本** — installer 接受 tag 参数:
```bash
curl -fsSL https://raw.githubusercontent.com/Bwkyd/url-md/main/install.sh | bash -s v0.1.2
```
</details>

## 用法

```bash
url-md <URL>              # 输出 Markdown 到 stdout(不下图)
url-md <URL> -o out/      # 存到目录 + 自动下图到 out/assets/
```

其他 flag 见 `url-md --help`:`--no-assets` 关闭下图 · `--assets <DIR>` 自定义图片目录 · `--verbose / --quiet` · `--timeout`。

**退出码**: 0=成功 · 10=网络 · 11=反爬 · 12=付费墙 · 13=登录墙 · 20=解析 · 30=IO · 99=内部

## 抓出来长啥样

```bash
url-md https://mp.weixin.qq.com/s/AMJBh90iNEZBRLY3iWsYxQ -o out/
```

**文件 1**: `out/2026-04-17-mp_weixin_qq_com-畅销书是怎么浪费你时间的.md`

```markdown
---
title: 畅销书是怎么浪费你时间的？
author: Niklas Göke
publish_time: 2026年4月17日 07:42
cover_url: https://mmbiz.qpic.cn/.../0?wx_fmt=jpeg
extract_method: weixin
word_count: 3247
reading_time_minutes: 11
source_url: https://mp.weixin.qq.com/s/AMJBh90iNEZBRLY3iWsYxQ
source_adapter: weixin
fetched_at: 2026-04-17T16:17:48Z
---

**开智君说**

![图片](assets/img-0001.gif)

畅销书广受欢迎,但有必要读吗？本文作者对一本畅销书进行详细分析…
```

**文件 2**: `out/assets/img-0001.gif` … `img-0008.png`(8 张图全下载,Markdown 引用改为相对路径,**断网也能看**)

## 能抓什么

| 站点 | 支持度 |
|---|---|
| 微信公众号永久链 `mp.weixin.qq.com/s/*` | ✅ 完整(图 / 作者 / 发布时间 / 封面全齐) |
| HackerNews / Rust Book / 静态博客 | ✅ generic 兜底 |
| 多文章列表首页 | ✅ 合并所有 `<article>` |


## 状态

**v0.1.x · 只做单 URL 抓取**。批量 / HTTP / MCP / 登录墙在规划中。

## 贡献

新增适配器:`src/url-md-adapters/src/<site>.rs`(参考现有的 `weixin.rs`)。PR 请带 `Signed-off-by:`(DCO)。

## 许可

Apache-2.0 — see [LICENSE](./LICENSE)
