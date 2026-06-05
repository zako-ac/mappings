pub fn to_native_korean(number: u32) -> String {
    const UNITS: [&str; 10] = [
        "", "한", "두", "세", "네", "다섯", "여섯", "일곱", "여덟", "아홉",
    ];
    const TENS: [&str; 10] = [
        "", "열", "스물", "서른", "마흔", "쉰", "예순", "일흔", "여든", "아흔",
    ];
    const TENS_ALONE: [&str; 10] = [
        "", "열", "스무", "서른", "마흔", "쉰", "예순", "일흔", "여든", "아흔",
    ];

    let t = (number / 10) as usize;
    let u = (number % 10) as usize;

    match (t, u) {
        (0, 0) => "영".to_string(),
        (0, _) => UNITS[u].to_string(),
        (_, 0) => TENS_ALONE[t].to_string(),
        _ => format!("{}{}", TENS[t], UNITS[u]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_native_korean() {
        assert_eq!(to_native_korean(1), "한");
        assert_eq!(to_native_korean(2), "두");
        assert_eq!(to_native_korean(3), "세");
        assert_eq!(to_native_korean(4), "네");
        assert_eq!(to_native_korean(5), "다섯");
        assert_eq!(to_native_korean(6), "여섯");
        assert_eq!(to_native_korean(7), "일곱");
        assert_eq!(to_native_korean(8), "여덟");
        assert_eq!(to_native_korean(9), "아홉");
        assert_eq!(to_native_korean(10), "열");

        assert_eq!(to_native_korean(11), "열한");
        assert_eq!(to_native_korean(20), "스무");
        assert_eq!(to_native_korean(23), "스물세");
        assert_eq!(to_native_korean(30), "서른");
        assert_eq!(to_native_korean(45), "마흔다섯");
        assert_eq!(to_native_korean(99), "아흔아홉");
    }
}
