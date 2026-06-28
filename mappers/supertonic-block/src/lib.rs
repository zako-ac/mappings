use unicode_normalization::UnicodeNormalization;
use zako3_tts_matching_sdk::prelude::*;

pub fn process(input: Input) -> Output {
    let normalized: String = input.text.nfc().collect();

    let removed = remove_known(&normalized);
    let reduced = reduce_repeats(&removed);

    let whitespace = regex::Regex::new(r"\s+").unwrap();
    let result = whitespace.replace_all(&reduced, " ").trim().to_string();

    let is_garbage = result.is_empty()
        || (result.chars().count() > 4 && !result.chars().any(char::is_alphanumeric))
        || is_exclamation_noise(&removed);

    if is_garbage && !normalized.trim().is_empty() {
        // Return a space (not empty string): the host pipeline ignores empty
        // text ("no change"), but a space gets accepted and downstream .trim()
        // produces "" which causes the TTS to be skipped.
        return Output::text(" ").override_pipeline(vec![]);
    }
    Output::text(result.replace("*", " "))
}

fn remove_known(text: &str) -> String {
    let known = regex::Regex::new(r"[\^_=<>{}\[\]\\|`]+").unwrap();
    let without_known = known.replace_all(text, " ");
    let clusters = regex::Regex::new(r#"[!~"']{2,}"#).unwrap();
    clusters.replace_all(&without_known, " ").into_owned()
}

fn reduce_repeats(text: &str) -> String {
    let mut current: Vec<char> = text.chars().collect();
    loop {
        let next = reduce_once(&current);
        if next.len() == current.len() {
            return next.into_iter().collect();
        }
        current = next;
    }
}

fn reduce_once(chars: &[char]) -> Vec<char> {
    let n = chars.len();
    let mut out = Vec::with_capacity(n);
    let mut i = 0;
    while i < n {
        let c = chars[i];
        let mut j = i;
        while j < n && chars[j] == c {
            j += 1;
        }
        if j - i >= 3 {
            out.push(c);
            i = j;
            continue;
        }

        if i + 1 < n {
            let (a, b) = (chars[i], chars[i + 1]);
            let mut k = i;
            let mut count = 0;
            while k + 1 < n && chars[k] == a && chars[k + 1] == b {
                k += 2;
                count += 1;
            }
            if count >= 3 {
                out.push(a);
                out.push(b);
                i = k;
                continue;
            }
        }

        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Detect noise patterns like "아!ㅇ!아!ㅏ" where single exclamation marks are
/// interspersed between very few unique alphanumeric characters — a common
/// griefing/spam pattern that produces garbled TTS output.
fn is_exclamation_noise(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();

    // Count interspersed '!' marks: alphanumeric ! alphanumeric
    let mut interspersed = 0u32;
    for i in 1..n.saturating_sub(1) {
        if chars[i] == '!' && chars[i - 1].is_alphanumeric() && chars[i + 1].is_alphanumeric() {
            interspersed += 1;
        }
    }

    if interspersed < 3 {
        return false;
    }

    // Count unique alphanumeric characters
    let mut unique: Vec<char> = Vec::new();
    for &c in &chars {
        if c.is_alphanumeric() && !unique.contains(&c) {
            unique.push(c);
        }
    }

    unique.len() <= 3
}

export_mapper!(process);

#[cfg(test)]
mod tests {
    use super::*;

    fn make_input(text: &str) -> Input {
        Input {
            text: text.to_string(),
            mapping_list: None,
            caller_info: None,
            mapper_list: None,
            guild_id: None,
            channel_id: None,
        }
    }

    /// Helper: returns true if the output blocks TTS (whitespace-only text + pipeline override).
    /// The mapper returns " " (space), which the host accepts; downstream .trim() → "" → TTS skipped.
    fn is_blocked(output: &Output) -> bool {
        output.text.trim().is_empty() && output.override_future_mappers == Some(vec![])
    }

    // ── Patterns that should be BLOCKED ──

    #[test]
    fn block_repeated_exclamations() {
        let out = process(make_input("!!!!!!!!!!!!!!!!"));
        assert!(is_blocked(&out), "should block: !!!!!!!!!!!!!!!!");
    }

    #[test]
    fn block_exclamation_tilde_mixed() {
        let out = process(make_input("!~!~!~!!~!~!~!"));
        assert!(is_blocked(&out), "should block: !~!~!~!!~!~!~!");
    }

    #[test]
    fn block_exclamation_tilde_alternating() {
        let out = process(make_input("!~!~!~!~!"));
        assert!(is_blocked(&out), "should block: !~!~!~!~!");
    }

    #[test]
    fn block_short_exclamation_tilde() {
        let out = process(make_input("!~!"));
        assert!(is_blocked(&out), "should block: !~!");
    }

    #[test]
    fn block_two_exclamations() {
        let out = process(make_input("!!"));
        assert!(is_blocked(&out), "should block: !!");
    }

    #[test]
    fn block_three_exclamations() {
        let out = process(make_input("!!!"));
        assert!(is_blocked(&out), "should block: !!!");
    }

    #[test]
    fn block_repeated_tildes() {
        let out = process(make_input("~~~~"));
        assert!(is_blocked(&out), "should block: ~~~~");
    }

    #[test]
    fn block_exclamation_tilde_pair() {
        let out = process(make_input("!~"));
        assert!(is_blocked(&out), "should block: !~");
    }

    #[test]
    fn block_quotes_cluster() {
        let out = process(make_input("'''''''"));
        assert!(is_blocked(&out), "should block: '''''''");
    }

    #[test]
    fn block_exclamations_with_spaces() {
        let out = process(make_input("!! !! !!"));
        assert!(is_blocked(&out), "should block: !! !! !!");
    }

    // ── Patterns that should NOT be blocked ──

    #[test]
    fn pass_single_exclamation() {
        let out = process(make_input("!"));
        assert!(!is_blocked(&out), "single ! should not be blocked");
        assert_eq!(out.text, "!");
    }

    #[test]
    fn pass_normal_text_with_exclamations() {
        let out = process(make_input("Hello!!!"));
        assert!(!is_blocked(&out), "Hello!!! should not be blocked");
        assert_eq!(out.text, "Hello");
    }

    #[test]
    fn pass_korean_text() {
        let out = process(make_input("안녕하세요"));
        assert!(!is_blocked(&out), "Korean text should not be blocked");
        assert_eq!(out.text, "안녕하세요");
    }

    #[test]
    fn pass_korean_with_exclamations() {
        let out = process(make_input("안녕!!!"));
        assert!(!is_blocked(&out), "안녕!!! should not be blocked");
        assert_eq!(out.text, "안녕");
    }

    #[test]
    fn pass_mixed_text() {
        let out = process(make_input("Hello! World!"));
        assert!(!is_blocked(&out), "Hello! World! should not be blocked");
        assert_eq!(out.text, "Hello! World!");
    }

    #[test]
    fn pass_empty_string() {
        let out = process(make_input(""));
        assert!(!is_blocked(&out), "empty string should not be 'blocked'");
        assert_eq!(out.text, "");
    }

    #[test]
    fn pass_whitespace_only() {
        let out = process(make_input("   "));
        assert!(!is_blocked(&out), "whitespace-only should not be 'blocked'");
        assert_eq!(out.text, "");
    }

    #[test]
    fn pass_exclamations_around_text() {
        let out = process(make_input("!!!Hello!!!"));
        assert!(!is_blocked(&out), "!!!Hello!!! should not be blocked");
        assert_eq!(out.text, "Hello");
    }

    // ── Korean exclamation noise (should be BLOCKED) ──

    #[test]
    fn block_korean_excl_noise_short() {
        let out = process(make_input("아!ㅇ!아!ㅏ"));
        assert!(is_blocked(&out), "should block: 아!ㅇ!아!ㅏ");
    }

    #[test]
    fn block_korean_excl_noise_long() {
        let text = "아!ㅇ!아!!ㅏㅏㅇ!!아!아!ㅇ!아!!ㅏㅏㅇ!!아!아!ㅇ!아!!ㅏㅏㅇ!!아!아!ㅇ!아!!ㅏㅏㅇ!!아!아!ㅇ!아!!ㅏㅏㅇ!!아!아!ㅇ!아!!ㅏㅏㅇ!!아!아!ㅇ!아!!ㅏㅏㅇ!!아!";
        let out = process(make_input(text));
        assert!(
            is_blocked(&out),
            "should block: long Korean exclamation noise"
        );
    }

    #[test]
    fn block_korean_excl_noise_repeated_syllable() {
        let out = process(make_input("와!와!와!와!"));
        assert!(is_blocked(&out), "should block: 와!와!와!와!");
    }

    // ── Korean exclamation noise boundary (should NOT be blocked) ──

    #[test]
    fn pass_korean_single_exclamation() {
        let out = process(make_input("안녕!"));
        assert!(!is_blocked(&out), "안녕! should not be blocked");
    }

    #[test]
    fn pass_korean_two_words_with_exclamations() {
        let out = process(make_input("안녕! 반가워!"));
        assert!(!is_blocked(&out), "안녕! 반가워! should not be blocked");
    }

    #[test]
    fn pass_korean_many_unique_with_exclamations() {
        // 5 unique syllables — above the ≤4 threshold
        let out = process(make_input("진짜!대박!진짜!대박!"));
        assert!(
            !is_blocked(&out),
            "진짜!대박!진짜!대박! should not be blocked"
        );
    }

    #[test]
    fn pass_english_exclamation_between_words() {
        let out = process(make_input("Hello!World!Go!"));
        assert!(!is_blocked(&out), "Hello!World!Go! should not be blocked");
    }

    // ── reduce_repeats tests ──

    #[test]
    fn reduce_repeated_chars() {
        let out = process(make_input("wwwwwwhat"));
        assert!(!is_blocked(&out));
        assert_eq!(out.text, "what");
    }

    #[test]
    fn reduce_alternating_pairs() {
        let out = process(make_input("hahahahahaha"));
        assert!(!is_blocked(&out));
        assert_eq!(out.text, "ha");
    }
}
