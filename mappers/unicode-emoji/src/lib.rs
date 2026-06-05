use std::borrow::Cow;

use unicode_segmentation::UnicodeSegmentation;
use zako3_tts_matching_sdk::prelude::*;

include!(concat!(env!("OUT_DIR"), "/emoji_map.rs"));

pub fn process(input: Input) -> Output {
    let mut out = String::with_capacity(input.text.len());

    for grapheme in input.text.graphemes(true) {
        let key: Cow<str> = if grapheme.contains('\u{FE0F}') {
            Cow::Owned(grapheme.chars().filter(|&c| c != '\u{FE0F}').collect())
        } else {
            Cow::Borrowed(grapheme)
        };

        match EMOJI_READINGS.binary_search_by(|(k, _)| (*k).cmp(key.as_ref())) {
            Ok(i) => out.push_str(EMOJI_READINGS[i].1),
            Err(_) => out.push_str(grapheme),
        }
    }

    Output::text(out)
}

export_mapper!(process);

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str) -> String {
        process(Input {
            text: text.to_string(),
            mapping_list: None,
            caller_info: None,
            mapper_list: None,
            guild_id: None,
            channel_id: None,
        })
        .text
    }

    #[test]
    fn base_emoji() {
        assert_eq!(run("😀"), "활짝 웃는 얼굴");
    }

    #[test]
    fn skin_tone_modifier() {
        assert_eq!(run("👋🏻"), "흔드는 손: 하얀 피부");
    }

    #[test]
    fn zwj_family_sequence() {
        assert_eq!(run("👨‍👩‍👦"), "가족: 남자 여자 남자 아이");
    }

    #[test]
    fn variation_selector_normalized() {
        assert_eq!(run("❤"), run("❤️"));
        assert_eq!(run("❤"), "빨간색 하트");
    }

    #[test]
    fn country_flag() {
        assert_eq!(run("🇰🇷"), "깃발: 대한민국");
        assert_eq!(run("🇺🇸"), "깃발: 미국");
    }

    #[test]
    fn subdivision_tag_flag() {
        assert_eq!(run("🏴󠁧󠁢󠁥󠁮󠁧󠁿"), "깃발: 잉글랜드");
    }

    #[test]
    fn mixed_text_preserves_punctuation_and_spacing() {
        assert_eq!(run("안녕!😀?"), "안녕!활짝 웃는 얼굴?");
    }

    #[test]
    fn non_emoji_passthrough() {
        assert_eq!(run("Hello, 안녕하세요!"), "Hello, 안녕하세요!");
    }
}
