use std::collections::HashMap;
use zako3_tts_matching_sdk::prelude::*;

pub fn process(input: Input) -> Output {
    if let Some(mapping_list) = input.mapping_list {
        let text = mapping_list
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
            });

        let id_map: HashMap<&str, &str> = mapping_list
            .emoji_rules
            .iter()
            .map(|r| (r.emoji_id.as_str(), r.replacement.as_str()))
            .collect();
        let name_map: HashMap<&str, &str> = mapping_list
            .emoji_rules
            .iter()
            .map(|r| (r.emoji_name.as_str(), r.replacement.as_str()))
            .collect();

        let emoji_re = regex::Regex::new(r"<a?:([^:>]+):(\d+)>").unwrap();
        let result = emoji_re
            .replace_all(&text, |caps: &regex::Captures| {
                let name = &caps[1];
                let id = &caps[2];
                id_map
                    .get(id)
                    .or_else(|| name_map.get(name))
                    .copied()
                    .unwrap_or("Emoji")
                    .to_string()
            })
            .to_string();

        Output::text(result)
    } else {
        Output::text(input.text)
    }
}

export_mapper!(process);
