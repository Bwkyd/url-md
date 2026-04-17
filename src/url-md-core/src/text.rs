//! 文本统计工具: 字数 + 阅读时间估算.
//!
//! 混合中英文场景:中文按字符数,英文按单词数(空格分词),两者相加.

/// 统计 Markdown 正文字数.
///
/// 策略:
/// - 中文 / 日文 / 韩文字符各算 1 个字
/// - 英文按空白分词,非空 token 算 1 个单词
/// - 忽略代码块内容(``` 之间)和 YAML frontmatter
pub fn count_words(body: &str) -> usize {
    let stripped = strip_code_blocks(body);
    let mut cjk_chars = 0usize;
    let mut ascii_words = 0usize;
    let mut in_ascii_word = false;

    for ch in stripped.chars() {
        if is_cjk(ch) {
            cjk_chars += 1;
            in_ascii_word = false;
        } else if ch.is_ascii_alphanumeric() {
            if !in_ascii_word {
                ascii_words += 1;
                in_ascii_word = true;
            }
        } else {
            in_ascii_word = false;
        }
    }

    cjk_chars + ascii_words
}

/// 基于字数估算阅读时间(分钟). 阅读速度参考:中文 ~300 字/分,英文 ~200 wpm.
/// 简化统一按 300 字/分 向上取整, 最少 1 分钟.
pub fn reading_time_minutes(word_count: usize) -> usize {
    const WPM: usize = 300;
    if word_count == 0 {
        return 0;
    }
    word_count.div_ceil(WPM)
}

fn is_cjk(ch: char) -> bool {
    matches!(ch as u32,
        0x4E00..=0x9FFF |   // CJK Unified Ideographs
        0x3400..=0x4DBF |   // CJK Ext A
        0x3040..=0x309F |   // Hiragana
        0x30A0..=0x30FF |   // Katakana
        0xAC00..=0xD7AF     // Hangul Syllables
    )
}

/// 去掉 ``` 围栏的代码块(粗略,只看行首).
fn strip_code_blocks(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut in_code = false;
    for line in body.lines() {
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if !in_code {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_pure_cjk() {
        assert_eq!(count_words("你好世界"), 4);
        // 认知科学从行动之环出发 = 11 个中文字符(标点不计)
        assert_eq!(count_words("认知科学,从「行动之环」出发"), 11);
    }

    #[test]
    fn counts_pure_ascii() {
        assert_eq!(count_words("hello world"), 2);
        assert_eq!(count_words("one two three four five"), 5);
    }

    #[test]
    fn counts_mixed() {
        assert_eq!(count_words("AI 时代来了"), 1 + 4); // "AI" + 4 中文
    }

    #[test]
    fn ignores_fenced_code_block() {
        let md = "hello\n```rust\nfn main() {}\n```\nworld";
        assert_eq!(count_words(md), 2); // hello + world
    }

    #[test]
    fn reading_time_boundaries() {
        assert_eq!(reading_time_minutes(0), 0);
        assert_eq!(reading_time_minutes(1), 1);
        assert_eq!(reading_time_minutes(300), 1);
        assert_eq!(reading_time_minutes(301), 2);
        assert_eq!(reading_time_minutes(900), 3);
    }
}
