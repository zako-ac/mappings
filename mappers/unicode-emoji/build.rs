use std::collections::BTreeMap;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::reader::Reader;
use unicode_properties::UnicodeEmoji;

const SOURCES: &[&str] = &["annotations/ko.xml", "annotations/ko_derived.xml"];

fn strip_fe0f(s: &str) -> String {
    s.chars().filter(|&c| c != '\u{FE0F}').collect()
}

fn is_regional_indicator(c: char) -> bool {
    matches!(c as u32, 0x1F1E6..=0x1F1FF)
}

fn is_emoji_cp(cp: &str) -> bool {
    let mut chars = cp.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => (c as u32) >= 0x80 && c.is_emoji_char(),
        (Some(_), Some(_)) => cp
            .chars()
            .any(|c| c.is_emoji_char() || is_regional_indicator(c)),
        _ => false,
    }
}

fn parse_into(path: &str, map: &mut BTreeMap<String, String>) {
    let content =
        fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
    let mut reader = Reader::from_str(&content);

    let mut pending_cp: Option<String> = None;
    let mut in_tts = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if e.name().as_ref() == b"annotation" => {
                let mut cp: Option<String> = None;
                let mut is_tts = false;
                for attr in e.attributes().flatten() {
                    let is_cp = attr.key.as_ref() == b"cp";
                    let is_type = attr.key.as_ref() == b"type";
                    let value = attr.unescape_value().unwrap();
                    if is_cp {
                        cp = Some(value.into_owned());
                    } else if is_type {
                        is_tts = value == "tts";
                    }
                }
                in_tts = is_tts;
                pending_cp = if is_tts { cp } else { None };
            }
            Ok(Event::Text(e)) if in_tts => {
                if let Some(cp) = pending_cp.take() {
                    let reading = e.unescape().unwrap().trim().to_string();
                    let key = strip_fe0f(&cp);
                    if !reading.is_empty() && is_emoji_cp(&key) {
                        map.insert(key, reading);
                    }
                }
                in_tts = false;
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"annotation" => {
                in_tts = false;
                pending_cp = None;
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("xml parse error in {path}: {e}"),
            _ => {}
        }
    }
}

fn main() {
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    for src in SOURCES {
        println!("cargo:rerun-if-changed={src}");
        parse_into(src, &mut map);
    }

    let mut generated = String::new();
    generated.push_str("pub static EMOJI_READINGS: &[(&str, &str)] = &[\n");
    for (k, v) in &map {
        writeln!(generated, "    ({k:?}, {v:?}),").unwrap();
    }
    generated.push_str("];\n");

    let out_dir = env::var("OUT_DIR").unwrap();
    fs::write(Path::new(&out_dir).join("emoji_map.rs"), generated).unwrap();
}
