//! M1 Spike: reqwest 快路能抓几个 URL?
//!
//! 验证假设: 不少网页(尤其微信永久链)是服务端渲染好的静态 HTML,
//! reqwest + 普通 Chrome UA 就能拿到. 只有真正被反爬拦截或 JS 渲染
//! 的站点才需要回退到 CDP(agent_browser lib).
//!
//! 跑法: cargo run -p url-md-spike

use reqwest::blocking::Client;
use std::time::{Duration, Instant};

const UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

/// 每个 URL 带一个"内容存在标记"——用于判断是真抓到内容,还是抓到反爬拦截页.
struct TestCase {
    name: &'static str,
    url: &'static str,
    /// HTML 里必须出现的子串(缺了就不算成功,哪怕 HTTP 200).
    content_marker: &'static str,
}

const CASES: &[TestCase] = &[
    TestCase {
        name: "weixin_permanent",
        url: "https://mp.weixin.qq.com/s/AMJBh90iNEZBRLY3iWsYxQ",
        content_marker: "id=\"js_content\"",
    },
    TestCase {
        name: "hackernews",
        url: "https://news.ycombinator.com/",
        content_marker: "Hacker News",
    },
    TestCase {
        name: "rust_book",
        url: "https://doc.rust-lang.org/book/title-page.html",
        content_marker: "The Rust Programming Language",
    },
];

#[derive(Debug)]
enum Outcome {
    Ok { bytes: usize, ms: u128 },
    HttpError(u16),
    Blocked { bytes: usize, reason: String },
    Network(String),
}

fn try_http(client: &Client, case: &TestCase) -> Outcome {
    let t0 = Instant::now();
    let resp = match client.get(case.url).send() {
        Ok(r) => r,
        Err(e) => return Outcome::Network(e.to_string()),
    };
    let status = resp.status();
    if !status.is_success() {
        return Outcome::HttpError(status.as_u16());
    }
    let html = match resp.text() {
        Ok(s) => s,
        Err(e) => return Outcome::Network(format!("body: {e}")),
    };
    let ms = t0.elapsed().as_millis();

    // 关键: 内容完整性校验(避免"HTTP 200 但实际是拦截页"的假成功)
    if !html.contains(case.content_marker) {
        return Outcome::Blocked {
            bytes: html.len(),
            reason: format!("missing marker `{}`", case.content_marker),
        };
    }
    Outcome::Ok { bytes: html.len(), ms }
}

fn main() {
    let client = Client::builder()
        .user_agent(UA)
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .expect("build http client");

    println!("== M1 Spike: reqwest 快路成功率 ==");
    println!();
    let mut ok = 0;
    let mut blocked = 0;
    let mut failed = 0;

    for case in CASES {
        print!("[{}]  ", case.name);
        match try_http(&client, case) {
            Outcome::Ok { bytes, ms } => {
                println!("OK      bytes={bytes:<7} ms={ms}");
                ok += 1;
            }
            Outcome::HttpError(code) => {
                println!("HTTP    status={code}");
                failed += 1;
            }
            Outcome::Blocked { bytes, reason } => {
                println!("BLOCKED bytes={bytes:<7} reason={reason}");
                blocked += 1;
            }
            Outcome::Network(err) => {
                println!("NET     err={err}");
                failed += 1;
            }
        }
    }

    println!();
    println!(
        "Summary: {ok} ok / {blocked} blocked / {failed} failed  (total {})",
        CASES.len()
    );
    let rate = (ok as f32) / (CASES.len() as f32) * 100.0;
    println!("快路成功率: {rate:.0}%");

    if blocked > 0 || failed > 0 {
        println!();
        println!(
            "下一步: 为 blocked/failed 的 URL 实现 CDP 回退(use agent_browser;)"
        );
    }
}
