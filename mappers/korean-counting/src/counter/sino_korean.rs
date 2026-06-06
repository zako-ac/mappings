const DIGITS: [&str; 10] = ["", "일", "이", "삼", "사", "오", "육", "칠", "팔", "구"];
const PLACES: [&str; 4] = ["", "십", "백", "천"];
const GROUPS: [&str; 18] = [
    "",
    "만",
    "억",
    "조",
    "경",
    "해",
    "자",
    "양",
    "구",
    "간",
    "정",
    "재",
    "극",
    "항하사",
    "아승기",
    "나유타",
    "불가사의",
    "무량대수",
];

pub fn to_sino_korean(number: u32) -> String {
    to_sino_korean_str(&number.to_string())
}

/// Converts a string of decimal digits to sino-Korean, supporting magnitudes up
/// to 무량대수 (10^68). Non-digit characters and commas are ignored.
pub fn to_sino_korean_str(digits: &str) -> String {
    let cleaned: String = digits.chars().filter(char::is_ascii_digit).collect();
    let trimmed = cleaned.trim_start_matches('0');
    if trimmed.is_empty() {
        return "영".to_string();
    }

    let len = trimmed.len();
    let num_groups = len.div_ceil(4);
    let mut parts: Vec<String> = Vec::new();

    for gi in (0..num_groups).rev() {
        let end = len - 4 * gi;
        let start = end.saturating_sub(4);
        let group_str = group_to_korean(&trimmed[start..end]);
        if !group_str.is_empty() {
            parts.push(group_str);
            parts.push(GROUPS.get(gi).copied().unwrap_or("").to_string());
        }
    }

    parts.concat()
}

fn group_to_korean(group: &str) -> String {
    let glen = group.len();
    let mut s = String::new();
    for (idx, c) in group.chars().enumerate() {
        let d = c.to_digit(10).unwrap_or(0) as usize;
        if d == 0 {
            continue;
        }
        let place = glen - 1 - idx;
        if d == 1 && place > 0 {
            s.push_str(PLACES[place]);
        } else {
            s.push_str(DIGITS[d]);
            s.push_str(PLACES[place]);
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_sino_korean() {
        assert_eq!(to_sino_korean(0), "영");
        assert_eq!(to_sino_korean(1), "일");
        assert_eq!(to_sino_korean(2), "이");
        assert_eq!(to_sino_korean(3), "삼");
        assert_eq!(to_sino_korean(4), "사");
        assert_eq!(to_sino_korean(5), "오");
        assert_eq!(to_sino_korean(6), "육");
        assert_eq!(to_sino_korean(7), "칠");
        assert_eq!(to_sino_korean(8), "팔");
        assert_eq!(to_sino_korean(9), "구");
        assert_eq!(to_sino_korean(10), "십");

        assert_eq!(to_sino_korean(11), "십일");
        assert_eq!(to_sino_korean(20), "이십");
        assert_eq!(to_sino_korean(23), "이십삼");
        assert_eq!(to_sino_korean(30), "삼십");
        assert_eq!(to_sino_korean(45), "사십오");
        assert_eq!(to_sino_korean(99), "구십구");
    }

    #[test]
    fn test_to_sino_korean_groups() {
        assert_eq!(to_sino_korean(100), "백");
        assert_eq!(to_sino_korean(1000), "천");
        assert_eq!(to_sino_korean(10000), "일만");
        assert_eq!(to_sino_korean(12345), "일만이천삼백사십오");
        assert_eq!(to_sino_korean(100000000), "일억");
        assert_eq!(to_sino_korean(1234567890), "십이억삼천사백오십육만칠천팔백구십");
    }

    #[test]
    fn test_to_sino_korean_str_large() {
        assert_eq!(to_sino_korean_str("0"), "영");
        assert_eq!(to_sino_korean_str("000123"), "백이십삼");
        assert_eq!(to_sino_korean_str("1000000000000"), "일조");
        assert_eq!(to_sino_korean_str("10000000000000000"), "일경");
        // 10^20 = 해
        assert_eq!(to_sino_korean_str("100000000000000000000"), "일해");
        // 10^68 = 무량대수
        let mut muryang = String::from("1");
        muryang.push_str(&"0".repeat(68));
        assert_eq!(to_sino_korean_str(&muryang), "일무량대수");
        // mixed groups across high magnitudes: 5 * 10^68 + 3
        let mut mixed = String::from("5");
        mixed.push_str(&"0".repeat(67));
        mixed.push('3');
        assert_eq!(to_sino_korean_str(&mixed), "오무량대수삼");
    }
}
