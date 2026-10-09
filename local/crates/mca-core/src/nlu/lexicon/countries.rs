/// Country lexicon: surface forms (RU/EN) → canonical English name stored in
/// the application. Keeping the canonical form in the data (and only the
/// Russian label at render time) means reports stay stable across UI locales.
/// When several countries appear the earliest mention wins, matching reading
/// order.
pub fn canonical_country(text: &str) -> Option<&'static str> {
    COUNTRIES
        .iter()
        .filter_map(|(canonical, aliases)| {
            aliases
                .iter()
                .filter_map(|alias| text.find(alias).map(|pos| (pos, *canonical)))
                .min_by_key(|(pos, _)| *pos)
        })
        .min_by_key(|(pos, _)| *pos)
        .map(|(_, canonical)| canonical)
}

pub const COUNTRIES: &[(&str, &[&str])] = &[
    ("Germany", &["германи", "germany", "deutschland", "de "]),
    ("Kazakhstan", &["казахстан", "kazakhstan", "kazakh"]),
    ("Russia", &["россия", "рф", "russia"]),
    ("China", &["китай", "china", "кита"]),
    ("Turkey", &["турци", "turkey", "turkiye", "türkiye"]),
    ("UAE", &["оаэ", "эмираты", "uae", "dubai", "дубай"]),
    ("Poland", &["польш", "poland"]),
    ("Italy", &["италья", "italy"]),
    ("Spain", &["испан", "spain"]),
    ("France", &["франц", "france"]),
    (
        "Netherlands",
        &["нидерланд", "голланд", "netherlands", "holland"],
    ),
    ("Lithuania", &["литв", "lithuania"]),
    ("Latvia", &["латви", "latvia"]),
    ("Estonia", &["эстони", "estonia"]),
    ("USA", &["сша", "usa", "сша"]),
    ("India", &["индия", "india"]),
    ("Japan", &["япони", "japan"]),
    ("South Korea", &["южная корея", "корея", "korea"]),
    ("UK", &["великобритани", "англи", "uk", "britain"]),
    ("Ukraine", &["украин", "ukraine"]),
    ("Belarus", &["беларус", "belarus"]),
    ("Uzbekistan", &["узбекистан", "uzbekistan"]),
    ("Kyrgyzstan", &["кыргызстан", "киргизия", "kyrgyzstan"]),
    ("Azerbaijan", &["азербайджан", "azerbaijan"]),
    ("Georgia", &["грузи", "georgia"]),
    ("Armenia", &["армени", "armenia"]),
    ("Czech Republic", &["чех", "czech"]),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_russian_to_canonical() {
        assert_eq!(canonical_country("из германии"), Some("Germany"));
        assert_eq!(canonical_country("в казахстан"), Some("Kazakhstan"));
    }

    #[test]
    fn maps_english_to_canonical() {
        assert_eq!(canonical_country("from germany"), Some("Germany"));
    }

    #[test]
    fn unknown_country_is_none() {
        assert_eq!(canonical_country("из атлантиды"), None);
    }
}
