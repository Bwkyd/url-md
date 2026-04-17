# url-md

> **任意 URL → 结构化 Markdown**。开源 Rust CLI,对标 [42md.cc](https://42md.cc/cli)。
> CLI 第一 · webapp 薄壳 · MCP 为服务模式之一。

## 状态

**0.1.0 · Phase 1 MVP**(2026-04)。

可用:
- 单二进制 `url-md` CLI
- `url-md md <url>` 子命令,输出带 YAML frontmatter 的 Markdown
- `weixin` / `generic` 两个适配器
- reqwest 快路(M1 Spike 实测微信永久链 / HackerNews / Rust Book 100% 命中)

未来(按 [spec](./spec/dev/cli-architecture.spec.md)):
- CdpFetcher (JS-heavy 站点 + 登录墙)
- `batch` / `serve --http` / `serve --mcp` / `login` 子命令
- 更多适配器 (zhihu / substack / github / twitter)

## 为什么

2025-11 起进入 AI 新时代。产品的第一消费者是 agent,不再是人。按阳志平"4 命令原则":一个产品值不值得做,看能否概括为 4 个极简 CLI 命令。

url-md 的 4 命令(规划中):

```
url-md md <url>            # 单 URL → Markdown(已实现)
url-md batch <source>      # 批量(待实现)
url-md serve --http|--mcp  # HTTP 或 MCP 服务(待实现)
url-md login <site>        # 站点 cookie 桥接,合法授权(待实现)
```

## 同一 Rust 二进制 · 多种入口

```
Terminal ─────────────────┐
WebApp (HTTP)──────────┐  │
Claude / Cursor (MCP)──┼──┼──→ url-md 二进制(Rust)
                       │  │        ↓
                       └──┴──→ pipeline::fetch_and_convert()
                                 ↓
                    Router → Adapter(weixin/generic/...)
                                 ↓
                    Fetcher(reqwest 快路 | CDP 回退)
                                 ↓
                    Parser (DOM → Markdown 保真)
```

- 三种入口共享同一核心 `fetch_and_convert()`,同 URL 输出字节级一致
- 反爬内核复用 [vercel-labs/agent-browser](https://github.com/vercel-labs/agent-browser)(Apache-2.0,已 fork 到 [Bwkyd/agent-browser](https://github.com/Bwkyd/agent-browser) 加 `[lib]` target)

## 快速上手

```bash
# 构建
git clone https://github.com/Bwkyd/url-md.git
cd url-md
cargo build --release

# 抓一篇微信永久链
./target/release/url-md md https://mp.weixin.qq.com/s/xxxxxxxx

# 输出到文件(自动命名 yyyy-mm-dd-host-slug.md)
mkdir -p out && ./target/release/url-md md https://mp.weixin.qq.com/s/xxxxxxxx -o out/
```

输出示例(YAML frontmatter + Markdown 正文):

```markdown
---
author: Niklas Göke
cover_url: https://mmbiz.qpic.cn/.../0?wx_fmt=jpeg
fetched_at: 2026-04-17T15:58:14Z
source_adapter: weixin
title: 畅销书是怎么浪费你时间的？
---

**开智君说**
畅销书广受欢迎,但有必要读吗？...
```

## 与 42md 的对比

| 维度 | 42md | url-md |
|---|---|---|
| 许可 | 闭源 | **Apache-2.0 开源** |
| 分发 | `curl \| bash` 私有 | `cargo install` / GitHub Release |
| 扩展 | 无 | Adapter 矩阵,社区可贡献 |
| 反爬 | Rust + CfT + 裸 CDP | **同栈**,复用 agent-browser lib |
| MCP 支持 | — | 规划中(单二进制 `serve --mcp`) |

## 项目结构

```
url-md/
├── crates/
│   ├── url-md-core/        # 无状态核心 (Fetcher / Adapter / Pipeline trait)
│   ├── url-md-adapters/    # weixin + generic 适配器
│   ├── url-md/             # CLI binary (clap + md 子命令)
│   └── url-md-spike/       # M1 Spike 快路成功率验证(过渡 crate)
├── .42cog/                 # 认知敏捷法档案(meta/real/cog)
│   ├── meta/meta.md        # 项目元信息 + 4 命令自检
│   ├── real/real.md        # 5 条硬约束 + 3 可选约束
│   └── cog/cog.md          # 9 类实体认知模型
├── spec/dev/               # 开发规约
│   ├── cli-architecture.spec.md       # 完整 CLI 架构 v0.2.1
│   ├── agent-browser-lib.spec.md      # agent-browser fork 改造 v0.1.2
│   └── scripts/            # 自动化校验脚本
├── Cargo.toml              # workspace
└── LICENSE                 # Apache-2.0
```

## 起源

本项目起源于 Python MCP [Bwkyd/wexin-read-mcp](https://github.com/Bwkyd/wexin-read-mcp)。
微信加强反爬后,决定用 Rust 重写,并把范围从"微信专用"扩到"任意 URL"。
老 repo(Python)继续维护微信场景并已在 v0.2.0 切到 agent-browser 内核。
通用场景请用 url-md。

## 开发

```bash
# 全量测试
cargo test --workspace

# 校验 Phase 1 交付
bash spec/dev/scripts/verify-url-md-core-impl.sh --with-e2e

# 端到端 HN 抓取(网络依赖)
cargo run --release -p url-md -- md https://news.ycombinator.com
```

贡献者请先读 [`spec/dev/cli-architecture.spec.md`](./spec/dev/cli-architecture.spec.md)。所有 PR 需带 `Signed-off-by:`(DCO),新 adapter 需提交至少 2 个 golden fixture。

## 许可

Apache-2.0 · see [LICENSE](./LICENSE)
