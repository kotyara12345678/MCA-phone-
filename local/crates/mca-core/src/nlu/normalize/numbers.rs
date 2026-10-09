//! Number handling: Russian numeral words folded to digits, and digit scanning.

/// Russian numeral words 1..100. Multi-word forms ("двадцать пять") are folded
/// by repeated application of [`word_to_number`] during the scan below.
const NUMBER_WORDS: &[(&str, u32)] = &[
    ("один", 1),
    ("одна", 1),
    ("одну", 1),
    ("два", 2),
    ("две", 2),
    ("три", 3),
    ("четыре", 4),
    ("пять", 5),
    ("шесть", 6),
    ("семь", 7),
    ("восемь", 8),
    ("девять", 9),
    ("десять", 10),
    ("одиннадцать", 11),
    ("двенадцать", 12),
    ("тринадцать", 13),
    ("четырнадцать", 14),
    ("пятнадцать", 15),
    ("шестнадцать", 16),
    ("семнадцать", 17),
    ("восемнадцать", 18),
    ("девятнадцать", 19),
    ("двадцать", 20),
    ("тридцать", 30),
    ("сорок", 40),
    ("пятьдесят", 50),
    ("шестьдесят", 60),
    ("семьдесят", 70),
    ("восемьдесят", 80),
    ("девяносто", 90),
    ("сто", 100),
];

fn word_to_number(word: &str) -> Option<u32> {
    NUMBER_WORDS
        .iter()
        .find(|(w, _)| *w == word)
        .map(|(_, n)| *n)
}

pub fn fold_number_words(text: &str) -> String {
    let words: Vec<&str> = text.split(' ').filter(|w| !w.is_empty()).collect();
    let mut out: Vec<String> = Vec::with_capacity(words.len());
    let mut i = 0;
    while i < words.len() {
        // "двадцать пять" -> 25; "сто двадцать" stays out of scope for MVP
        // weights and simply yields two tokens.
        match pair_value(words[i], words.get(i + 1)) {
            Some(value) => {
                out.push(value.to_string());
                i += 2;
            }
            None => {
                match word_to_number(words[i]) {
                    Some(n) => out.push(n.to_string()),
                    None => out.push(words[i].to_string()),
                }
                i += 1;
            }
        }
    }
    out.join(" ")
}

fn pair_value(first: &str, second: Option<&&str>) -> Option<u32> {
    let tens = word_to_number(first)?;
    let ones = word_to_number(second?)?;
    ((20..100).contains(&tens) && ones <= 9).then_some(tens + ones)
}

/// Extracts the first decimal number in the text, accepting both `,` and `.`
/// as the decimal separator (speech-to-text emits either).
pub fn first_number(text: &str) -> Option<f64> {
    let mut current = String::new();
    for ch in text.chars().chain(std::iter::once(' ')) {
        if ch.is_ascii_digit() || ch == ',' || (ch == '.' && !current.is_empty()) {
            current.push(if ch == ',' { '.' } else { ch });
        } else if !current.is_empty() {
            if let Ok(value) = current.parse::<f64>() {
                return Some(value);
            }
            current.clear();
        }
    }
    None
}
