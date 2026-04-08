use zako3_tts_matching_sdk::prelude::*;

pub fn process(input: Input) -> Output {
    if let Some(mapping_list) = input.mapping_list {
        let result = mapping_list
            .text_rules
            .iter()
            .fold(input.text, |text, rule| {
                let case_sensitive = rule.case_sensitive;
                let re = if case_sensitive {
                    regex::RegexBuilder::new(&rule.pattern)
                        .case_insensitive(false)
                        .build()
                } else {
                    regex::RegexBuilder::new(&rule.pattern)
                        .case_insensitive(true)
                        .build()
                };

                match re {
                    Ok(re) => re.replace_all(&text, rule.replacement.as_str()).to_string(),
                    Err(_) => text,
                }
            })
            .to_string();

        Output::text(result)
    } else {
        Output::text(input.text)
    }
}

export_mapper!(process);
