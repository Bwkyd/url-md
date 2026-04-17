# url-md

**任意 URL → 干净 Markdown**。Rust 单二进制,开源。

```bash
cargo build --release
./target/release/url-md https://mp.weixin.qq.com/s/xxxxxxxx -o out/
```

得到:
- `out/YYYY-MM-DD-host-slug.md` — Markdown(含 YAML frontmatter)
- `out/assets/img-NNNN.{jpg,png,gif,webp}` — 图片全下载
- Markdown 里图片引用改写为相对路径,**离线可用**

## 用法

```bash
url-md <URL>              # 输出 Markdown 到 stdout(不下图)
url-md <URL> -o out/      # 存到目录 + 自动下图到 out/assets/
```

其他 flag 见 `url-md --help`:`--no-assets` 关闭下图 · `--verbose / --quiet` 日志级别 · `--assets <DIR>` 自定义图片目录 · `--timeout` 超时。

**退出码**: 0=成功 · 10=网络 · 11=反爬 · 12=付费墙 · 13=登录墙 · 20=解析 · 30=IO · 99=内部

## 现在能抓什么

| 站点 | 支持度 |
|---|---|
| 微信公众号永久链 `mp.weixin.qq.com/s/*` | ✅ 完整(含图 / 作者 / 发布时间 / 封面) |
| HackerNews / Rust Book / 静态博客 | ✅ generic 兜底 |
| 多文章列表首页 | ✅ 合并所有 `<article>` |

## 状态

**v0.1.0 · 当前只做单 URL 抓取**。批量 / HTTP 服务 / MCP / 登录墙等待后续版本。

## 许可

Apache-2.0 — see [LICENSE](./LICENSE)
