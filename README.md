# url-md

**任意 URL → 干净 Markdown**。Rust 单二进制,开源。

```bash
cargo build --release
./target/release/url-md https://mp.weixin.qq.com/s/xxxxxxxx
```

输出:带 YAML frontmatter 的 Markdown,正文 + 图片链接 + 元数据全齐。

## 装一个带上图片

```bash
url-md https://example.com/article -o out/ --assets out/assets/
```

- Markdown 写到 `out/YYYY-MM-DD-host-slug.md`
- 图片并发下载到 `out/assets/img-NNNN.{jpg,png,gif,webp}`
- Markdown 引用改写为相对路径,**彻底离线可用**

## 现在能抓什么

| 站点 | 支持度 |
|---|---|
| 微信公众号永久链 `mp.weixin.qq.com/s/*` | ✅ 完整(含图 / 作者 / 发布时间 / 封面) |
| HackerNews / Rust Book / 静态博客 | ✅ generic 兜底 |
| 多文章列表首页 | ✅ 合并所有 `<article>` |

## 用法

```bash
url-md <URL>                    # 最短: 别名形式
url-md md <URL>                 # 完整子命令
url-md <URL> -o out/            # 存到目录(自动命名)
url-md <URL> -o out/ --assets out/assets/   # 图片一起下载
url-md <URL> --verbose          # 详细过程输出
url-md <URL> --quiet            # 静默
```

**退出码**: 0=成功 · 10=网络 · 11=反爬 · 12=付费墙 · 13=登录墙 · 20=解析 · 30=IO · 99=内部

## 为什么

- 微信等中文站点抓取刚需,Python Playwright 方案被反爬打穿
- [42md.cc](https://42md.cc/cli) 不开源 + 有配额
- **我们:单二进制 · 快 · 无配额 · 本地 · 开源(Apache-2.0)**

## 对标

同一条微信 URL:

| 工具 | 耗时 | 许可 |
|---|---|---|
| 42md 0.3.12 | 3.62 s | 闭源,有配额 |
| **url-md 0.1.0** | **1.15 s (3.15× faster)** | Apache-2.0,无配额 |

## 状态

**v0.1.0 · 当前只做单 URL 抓取**。批量 / HTTP 服务 / MCP / 登录墙等待后续版本。

## 许可

Apache-2.0 — see [LICENSE](./LICENSE)
