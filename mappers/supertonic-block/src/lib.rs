use unicode_normalization::UnicodeNormalization;
use zako3_tts_matching_sdk::prelude::*;

pub fn process(input: Input) -> Output {
    let normalized: String = input.text.nfc().collect();
    let capped = cap_char_runs(&normalized);

    let symbols = regex::Regex::new(r"[~^*_=<>{}\[\]\\|`]+").unwrap();
    let no_symbols = symbols.replace_all(&capped, " ");

    let collapsed = collapse_punct_runs(&no_symbols);

    let whitespace = regex::Regex::new(r"\s+").unwrap();
    let result = whitespace.replace_all(&collapsed, " ").trim().to_string();

    if !result.chars().any(char::is_alphanumeric) {
        return Output::text(String::new()).override_pipeline(vec![]);
    }
    Output::text(result)
}

fn cap_char_runs(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let mut j = i;
        while j < chars.len() && chars[j] == c {
            j += 1;
        }
        let keep = if j - i >= 3 { 1 } else { j - i };
        for _ in 0..keep {
            out.push(c);
        }
        i = j;
    }
    out
}

fn collapse_punct_runs(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut prev: Option<char> = None;
    for c in text.chars() {
        if matches!(c, '!' | '?' | '.' | ',') && Some(c) == prev {
            continue;
        }
        out.push(c);
        prev = Some(c);
    }
    out
}

export_mapper!(process);
