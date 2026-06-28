use unicode_normalization::UnicodeNormalization;
use zako3_tts_matching_sdk::prelude::*;

// ═══════════════════════════════════════════════════════════════
// Noise Detector Pipeline
// ═══════════════════════════════════════════════════════════════
//
// Each detector is an independent `fn(&str) -> bool` predicate.
// The pipeline runs all detectors — if ANY returns true the text
// is treated as noise and TTS is skipped.
//
// To add a new noise pattern:
//   1. Write a `fn(&str) -> bool` detector below
//   2. Add it to the DETECTORS array
//   3. Add tests

type Detector = fn(&str) -> bool;

const DETECTORS: &[Detector] = &[
    detect_punctuation_only,
    detect_low_diversity,
    detect_high_punctuation_ratio,
    detect_exclamation_noise,
];

fn is_noise(text: &str) -> bool {
    DETECTORS.iter().any(|d| d(text))
}

// ── Detectors ──

/// Pure punctuation/symbols — no alphanumeric content at all (min 2 chars).
/// Catches: "!!!!!!!!", "!~!~!~", "'''''''"
fn detect_punctuation_only(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.len() >= 2 && !trimmed.chars().any(|c| c.is_alphanumeric())
}

/// Very few unique alphanumeric chars (≤ 3) repeated many times (≥ 8 total)
/// with no word breaks (spaces) **and** at least some punctuation/symbols
/// mixed in.  Catches low-lexical-diversity spam like
/// "ㅎㅎㅎㅏㅏ!ㅎㅎㅎㅏㅏ!ㅎㅎㅎㅏㅏ!" (2 unique, 18 alphanumeric).
///
/// Pure alphanumeric repetition without punctuation (e.g. "hahahahahaha",
/// "ㅋㅋㅋㅋㅋ") is left to `reduce_repeats` — those are legitimate
/// onomatopoeia that reduce to a sensible TTS output.
fn detect_low_diversity(text: &str) -> bool {
    let alphanumeric: Vec<char> = text.chars().filter(|c| c.is_alphanumeric()).collect();
    if alphanumeric.len() < 8 {
        return false;
    }
    if text.contains(' ') {
        return false; // multiple words → likely legitimate
    }
    let has_punct = text
        .chars()
        .any(|c| !c.is_alphanumeric() && !c.is_whitespace());
    if !has_punct {
        return false; // pure repetition → handled by reduce_repeats
    }
    let mut unique = alphanumeric;
    unique.sort_unstable();
    unique.dedup();
    unique.len() <= 3
}

/// Less than 50 % alphanumeric content in a text of 20+ non-whitespace chars.
/// Catches punctuation-heavy noise like
/// "쁘!!-에!이!!ㅡ!-ㅇ!!ㅎ!!흫!이-흐!!!흐헿!ㅎ!ㅎ!!헤하!!핳ㅎㅎㅏ!"
/// (19 alphanumeric out of 43 total ≈ 44 %).
fn detect_high_punctuation_ratio(text: &str) -> bool {
    let chars: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
    if chars.len() < 20 {
        return false;
    }
    let alpha = chars.iter().filter(|c| c.is_alphanumeric()).count();
    let ratio = alpha as f64 / chars.len() as f64;
    ratio < 0.5
}

/// Single '!' marks interspersed between alphanumeric chars with ≤ 3 unique
/// alphanumeric characters.  Catches "아!ㅇ!아!ㅏ" (3 interspersed !, 3 unique).
fn detect_exclamation_noise(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();

    let mut interspersed = 0u32;
    for i in 1..n.saturating_sub(1) {
        if chars[i] == '!' && chars[i - 1].is_alphanumeric() && chars[i + 1].is_alphanumeric() {
            interspersed += 1;
        }
    }
    if interspersed < 3 {
        return false;
    }

    let mut unique: Vec<char> = chars
        .iter()
        .filter(|c| c.is_alphanumeric())
        .copied()
        .collect();
    unique.sort_unstable();
    unique.dedup();
    unique.len() <= 3
}

// ═══════════════════════════════════════════════════════════════
// Text Cleaning
// ═══════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════
// Entry Point
// ═══════════════════════════════════════════════════════════════

pub fn process(input: Input) -> Output {
    let normalized: String = input.text.nfc().collect();

    // Phase 1 — noise detection on raw text (before any cleaning)
    if !normalized.trim().is_empty() && is_noise(&normalized) {
        return Output::text(" ").override_pipeline(vec![]);
    }

    // Phase 2 — text cleaning
    let removed = remove_known(&normalized);
    let reduced = reduce_repeats(&removed);
    let whitespace = regex::Regex::new(r"\s+").unwrap();
    let result = whitespace.replace_all(&reduced, " ").trim().to_string();

    // Phase 3 — post-cleaning noise check
    if !normalized.trim().is_empty() && (result.is_empty() || is_noise(&result)) {
        return Output::text(" ").override_pipeline(vec![]);
    }

    Output::text(result.replace("*", " "))
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
    fn is_blocked(output: &Output) -> bool {
        output.text.trim().is_empty() && output.override_future_mappers == Some(vec![])
    }

    // ── Pure punctuation (should be BLOCKED) ──

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

    // ── Low-diversity noise (should be BLOCKED) ──

    #[test]
    fn block_korean_low_diversity_repeated_clusters() {
        let out = process(make_input("ㅎㅎㅎㅏㅏ!ㅎㅎㅎㅏㅏ!ㅎㅎㅎㅏㅏ!"));
        assert!(
            is_blocked(&out),
            "should block: ㅎㅎㅎㅏㅏ!ㅎㅎㅎㅏㅏ!ㅎㅎㅎㅏㅏ!"
        );
    }

    #[test]
    fn block_korean_low_diversity_single_char() {
        // Has punctuation (!) mixed in → low_diversity catches it
        let out = process(make_input("ㅋㅋㅋ!ㅋㅋㅋ!ㅋㅋㅋ!"));
        assert!(
            is_blocked(&out),
            "should block: ㅋ×9 with ! (low diversity)"
        );
    }

    // ── High punctuation ratio noise (should be BLOCKED) ──

    #[test]
    fn block_korean_high_punct_ratio() {
        let text = "쁘!!-에!이!!ㅡ!-ㅇ!!ㅎ!!흫!이-흐!!!흐헿!ㅎ!ㅎ!!헤하!!핳ㅎㅎㅏ!";
        let out = process(make_input(text));
        assert!(
            is_blocked(&out),
            "should block: Korean punctuation-heavy noise"
        );
    }

    // ── Normal text (should NOT be blocked) ──

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

    // ── Korean exclamation boundary (should NOT be blocked) ──

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
        // 4 unique syllables — above the ≤3 threshold
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

    // ── Low-diversity boundary (should NOT be blocked) ──

    #[test]
    fn pass_korean_short_laughter() {
        let out = process(make_input("ㅋㅋㅋ"));
        assert!(!is_blocked(&out), "ㅋㅋㅋ should not be blocked");
    }

    #[test]
    fn pass_korean_four_unique_no_space() {
        // 4 unique syllables, no punctuation → passes (reduce_repeats handles it)
        let out = process(make_input("진짜대박진짜대박"));
        assert!(!is_blocked(&out), "진짜대박진짜대박 should not be blocked");
    }

    // ── High punctuation ratio boundary (should NOT be blocked) ──

    #[test]
    fn pass_text_with_some_punctuation() {
        // 10 non-ws chars, 8 alphanumeric → 80% → pass
        let out = process(make_input("Wow!!! That's"));
        assert!(!is_blocked(&out), "Wow!!! That's should not be blocked");
    }

    #[test]
    fn pass_short_text_high_punct() {
        // < 10 non-ws chars → detector skips
        let out = process(make_input("!!Hi!!"));
        assert!(
            !is_blocked(&out),
            "!!Hi!! should not be blocked (too short)"
        );
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
