use zako3_tts_matching_sdk::prelude::*;

pub fn process(input: Input) -> Output {
    let re = regex::Regex::new(r"(?s)\|\|.*?\|\|").unwrap();
    Output::text(re.replace_all(&input.text, "스포일러").to_string())
}

export_mapper!(process);
