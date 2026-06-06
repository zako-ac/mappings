use zako3_tts_matching_sdk::prelude::*;

mod counter;
use counter::{to_native_korean, to_sino_korean, to_sino_korean_str};

// Ordered longest-first to ensure correct suffix matching
const NATIVE_COUNTERS: &[&str] = &[
    "켤레", "사람", "그릇", "포기", "송이", "군데", "가지", "마디",
    "바퀴", "차례", "두름", "모듬", "마리", "자루",
    "개", "살", "명", "번", "병", "잔", "권", "장",
    "대", "벌", "접", "톳", "손", "모", "단",
];

pub fn process(input: Input) -> Output {
    let text = input.text.trim();

    for &unit in NATIVE_COUNTERS {
        if let Some(rest) = text.strip_suffix(unit) {
            let num_str = rest.trim();
            if !num_str.is_empty() {
                if let Some(result) = convert(num_str, unit) {
                    return Output::text(result);
                }
            }
        }
    }

    Output::text(replace_numbers(text))
}

/// Replaces every number token in `text` with its sino-Korean reading. A number
/// token is a run of digits optionally containing comma thousands-separators and
/// a single decimal point (e.g. `123,456` or `1.5`).
fn replace_numbers(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut result = String::new();
    let mut i = 0;

    while i < chars.len() {
        if !chars[i].is_ascii_digit() {
            result.push(chars[i]);
            i += 1;
            continue;
        }

        let start = i;
        // Integer part: digits plus commas that sit between two digits.
        while i < chars.len() {
            if chars[i].is_ascii_digit() {
                i += 1;
            } else if chars[i] == ','
                && i + 1 < chars.len()
                && chars[i + 1].is_ascii_digit()
            {
                i += 1;
            } else {
                break;
            }
        }
        // Optional decimal part: a dot immediately followed by digits.
        if i + 1 < chars.len() && chars[i] == '.' && chars[i + 1].is_ascii_digit() {
            i += 1;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
        }

        let token: String = chars[start..i].iter().collect();
        result.push_str(&number_to_sino(&token));
    }

    result
}

fn number_to_sino(token: &str) -> String {
    let cleaned: String = token.chars().filter(|&c| c != ',').collect();
    let (int_part, frac_part) = match cleaned.find('.') {
        Some(pos) => (&cleaned[..pos], &cleaned[pos + 1..]),
        None => (cleaned.as_str(), ""),
    };

    let mut s = to_sino_korean_str(int_part);

    let frac = frac_part.trim_end_matches('0');
    if !frac.is_empty() {
        s.push_str("쩜");
        for c in frac.chars() {
            let d = c.to_digit(10).unwrap_or(0);
            s.push_str(&to_sino_korean(d));
        }
    }

    s
}

fn convert(num_str: &str, unit: &str) -> Option<String> {
    num_str.parse::<f64>().ok()?;

    let (int_str, frac_str) = match num_str.find('.') {
        Some(pos) => (&num_str[..pos], &num_str[pos + 1..]),
        None => (num_str, ""),
    };

    let integer: u32 = int_str.parse().ok()?;
    let frac_digits = frac_str.trim_end_matches('0');

    let has_decimal = !frac_digits.is_empty();

    let mut result = if has_decimal || integer > 99 {
        to_sino_korean(integer)
    } else {
        to_native_korean(integer)
    };

    if has_decimal {
        result.push_str("쩜");
        for c in frac_digits.chars() {
            let d = c.to_digit(10)? as u32;
            result.push_str(&to_sino_korean(d));
        }
    }

    result.push_str(unit);
    Some(result)
}

export_mapper!(process);

#[cfg(test)]
mod tests {
    use super::*;

    fn cv(s: &str, unit: &str) -> String {
        convert(s, unit).unwrap()
    }

    fn input(text: &str) -> Input {
        Input {
            text: text.to_string(),
            mapping_list: None,
            caller_info: None,
            mapper_list: None,
            guild_id: None,
            channel_id: None,
        }
    }

    #[test]
    fn test_integer_native() {
        assert_eq!(cv("1", "개"), "한개");
        assert_eq!(cv("2", "살"), "두살");
        assert_eq!(cv("5", "마리"), "다섯마리");
        assert_eq!(cv("10", "명"), "열명");
        assert_eq!(cv("23", "잔"), "스물세잔");
        assert_eq!(cv("99", "번"), "아흔아홉번");
    }

    #[test]
    fn test_integer_sino_large() {
        assert_eq!(cv("100", "개"), "백개");
        assert_eq!(cv("150", "장"), "백오십장");
        assert_eq!(cv("1000", "권"), "천권");
    }

    #[test]
    fn test_decimal_sino() {
        assert_eq!(cv("1.5", "개"), "일쩜오개");
        assert_eq!(cv("2.3", "살"), "이쩜삼살");
        assert_eq!(cv("10.75", "잔"), "십쩜칠오잔");
    }

    #[test]
    fn test_decimal_trailing_zero_stripped() {
        assert_eq!(cv("1.50", "개"), "일쩜오개");
        assert_eq!(cv("2.10", "명"), "이쩜일명");
    }

    #[test]
    fn test_whitespace_between_number_and_unit() {
        assert_eq!(process(input("3 개")).text, "세개");
        assert_eq!(process(input("1.5  마리")).text, "일쩜오마리");
    }

    #[test]
    fn test_no_match_passthrough() {
        assert_eq!(process(input("hello")).text, "hello");
        assert_eq!(process(input("abc개")).text, "abc개");
    }

    #[test]
    fn test_replace_comma_price() {
        assert_eq!(process(input("123,456원")).text, "십이만삼천사백오십육원");
        assert_eq!(process(input("가격은 1,000원입니다")).text, "가격은 천원입니다");
    }

    #[test]
    fn test_replace_multiple_numbers() {
        assert_eq!(
            process(input("3번 버스를 10분 기다렸다")).text,
            "삼번 버스를 십분 기다렸다"
        );
    }

    #[test]
    fn test_replace_decimal_in_text() {
        assert_eq!(process(input("무게는 1.5kg")).text, "무게는 일쩜오kg");
    }

    #[test]
    fn test_replace_preserves_non_numbers() {
        assert_eq!(process(input("hello")).text, "hello");
        assert_eq!(process(input("abc개")).text, "abc개");
    }

    #[test]
    fn test_replace_zero() {
        assert_eq!(process(input("재고 0")).text, "재고 영");
    }

    #[test]
    fn test_replace_dot_not_decimal() {
        assert_eq!(process(input("끝났다. 100개 남았다")).text, "끝났다. 백개 남았다");
    }

    #[test]
    fn test_all_units_match() {
        for &unit in NATIVE_COUNTERS {
            assert!(convert("3", unit).is_some(), "unit {unit} did not match");
        }
    }
}
