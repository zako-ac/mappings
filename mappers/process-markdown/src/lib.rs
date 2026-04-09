use zako3_tts_matching_sdk::prelude::*;

pub fn process(input: Input) -> Output {
    let strikethrough = regex::Regex::new(r"~~(.+?)~~").unwrap();
    let text = strikethrough.replace_all(&input.text, "$1").to_string();

    let link = regex::Regex::new(r"\[([^\]]+)\]\([^)]+\)").unwrap();
    let text = link.replace_all(&text, "링크와 함께 $1").to_string();

    Output::text(text)
}

export_mapper!(process);
