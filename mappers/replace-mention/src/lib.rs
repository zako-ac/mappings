use zako3_tts_matching_sdk::prelude::*;

pub fn process(input: Input) -> Output {
    let user_mention_re = regex::Regex::new(r"<@(\d+)>").unwrap();
    let channel_mention_re = regex::Regex::new(r"<#(\d+)>").unwrap();

    let result = user_mention_re
        .replace_all(&input.text, |caps: &regex::Captures| {
            let id = &caps[1];
            input
                .query_user(id)
                .and_then(|u| u.guild_nickname.or(u.global_nickname).or(Some(u.username)))
                .unwrap_or_else(|| "모름".to_string())
                + " 호출"
        })
        .to_string();

    let result = channel_mention_re
        .replace_all(&result, |caps: &regex::Captures| {
            let id = &caps[1];
            query_channel_raw(id)
                .map(|c| c.name)
                .unwrap_or_else(|| "채널".to_string())
        })
        .to_string();

    Output::text(result)
}

export_mapper!(process);
