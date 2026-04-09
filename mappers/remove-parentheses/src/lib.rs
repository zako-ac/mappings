use zako3_tts_matching_sdk::prelude::*;

pub fn process(input: Input) -> Output {
    let re = regex::Regex::new(r"\([^)]*\)").unwrap();
    Output::text(re.replace_all(&input.text, "").to_string())
}

export_mapper!(process);
