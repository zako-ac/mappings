use unicode_normalization::UnicodeNormalization;
use zako3_tts_matching_sdk::prelude::*;

pub fn process(input: Input) -> Output {
    let normalized: String = input.text.nfc().collect();

    let removed = remove_known(&normalized);
    let reduced = reduce_repeats(&removed);

    let whitespace = regex::Regex::new(r"\s+").unwrap();
    let result = whitespace.replace_all(&reduced, " ").trim().to_string();

    if result.chars().count() > 4 && !result.chars().any(char::is_alphanumeric) {
        return Output::text(String::new()).override_pipeline(vec![]);
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
