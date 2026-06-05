pub fn to_sino_korean(number: u32) -> String {
    if number == 0 {
        return "영".to_string();
    }

    const DIGITS: [&str; 10] = ["", "일", "이", "삼", "사", "오", "육", "칠", "팔", "구"];
    const PLACES: [&str; 4] = ["", "십", "백", "천"];
    const GROUPS: [&str; 3] = ["", "만", "억"];

    fn group_str(n: u32) -> String {
        let mut s = String::new();
        for i in (0..4u32).rev() {
            let d = (n / 10u32.pow(i)) % 10;
            if d == 0 {
                continue;
            }
            if d == 1 && i > 0 {
                s.push_str(PLACES[i as usize]);
            } else {
                s.push_str(DIGITS[d as usize]);
                s.push_str(PLACES[i as usize]);
            }
        }
        s
    }

    let group_values = [
        number % 10_000,
        (number / 10_000) % 10_000,
        number / 100_000_000,
    ];

    let mut parts: Vec<String> = group_values
        .iter()
        .enumerate()
        .filter(|&(_, v)| *v > 0)
        .map(|(i, &v)| format!("{}{}", group_str(v), GROUPS[i]))
        .collect();

    parts.reverse();
    parts.concat()
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
}
