use unicode_normalization::UnicodeNormalization;
use zako3_tts_matching_sdk::prelude::*;

pub fn process(input: Input) -> Output {
    let normalized: String = input.text.nfc().collect();

    let removed = remove_known(&normalized);
    let reduced = reduce_repeats(&removed);

    let whitespace = regex::Regex::new(r"\s+").unwrap();
    let result = whitespace.replace_all(&reduced, " ").trim().to_string();

    let is_garbage = result.is_empty()
        || (result.chars().count() > 4 && !result.chars().any(char::is_alphanumeric));

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
