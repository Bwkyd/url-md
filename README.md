# url-md

**任意 URL → 干净 Markdown**。Rust 单二进制,开源。

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
curl -fsSL https://raw.githubusercontent.com/Bwkyd/url-md/main/install.sh | bash -s v0.1.1
```
</details>

## 用法

```bash
url-md <URL>              # 输出 Markdown 到 stdout(不下图)
url-md <URL> -o out/      # 存到目录 + 自动下图到 out/assets/
```

产物:
- `out/YYYY-MM-DD-host-slug.md` — Markdown(含 YAML frontmatter)
- `out/assets/img-NNNN.{jpg,png,gif,webp}` — 图片全下载,Markdown 引用改为相对路径,**离线可用**

其他 flag 见 `url-md --help`:`--no-assets` 关闭下图 · `--assets <DIR>` 自定义图片目录 · `--verbose / --quiet` · `--timeout`。

**退出码**: 0=成功 · 10=网络 · 11=反爬 · 12=付费墙 · 13=登录墙 · 20=解析 · 30=IO · 99=内部

## 能抓什么

| 站点 | 支持度 |
|---|---|
| 微信公众号永久链 `mp.weixin.qq.com/s/*` | ✅ 完整(图 / 作者 / 发布时间 / 封面全齐) |
| HackerNews / Rust Book / 静态博客 | ✅ generic 兜底 |
| 多文章列表首页 | ✅ 合并所有 `<article>` |

## 状态

**v0.1.0 · 只做单 URL 抓取**。批量 / HTTP / MCP / 登录墙在规划中。

## 许可

Apache-2.0 — see [LICENSE](./LICENSE)
