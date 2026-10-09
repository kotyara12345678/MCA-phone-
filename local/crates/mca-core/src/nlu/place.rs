use super::lexicon::{cities, countries};

/// A place mention resolved to canonical names.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlaceMatch {
    pub country: Option<&'static str>,
    pub city: Option<&'static str>,
}

impl PlaceMatch {
    pub fn is_empty(&self) -> bool {
        self.country.is_none() && self.city.is_none()
    }
}

/// Resolves any country/city mention in the text. When both are present the
/// city determines the country when the country is unknown, and vice versa.
pub fn detect_place(text: &str) -> PlaceMatch {
    let country = countries::canonical_country(text);
    let city = cities::canonical_city(text);
    let country = country.or_else(|| city.and_then(country_for_city));
    PlaceMatch { country, city }
}

/// The city mention's country, when the lexicon knows it. Used to keep the
/// application consistent ("доставка в Алматы" implies Kazakhstan).
pub fn country_for_city(city: &str) -> Option<&'static str> {
    CITY_COUNTRY
        .iter()
        .find(|(c, _)| *c == city)
        .map(|(_, country)| *country)
}

const CITY_COUNTRY: &[(&str, &str)] = &[
    ("Almaty", "Kazakhstan"),
    ("Astana", "Kazakhstan"),
    ("Shymkent", "Kazakhstan"),
    ("Karaganda", "Kazakhstan"),
    ("Hamburg", "Germany"),
    ("Berlin", "Germany"),
    ("Munich", "Germany"),
    ("Frankfurt", "Germany"),
    ("Cologne", "Germany"),
    ("Stuttgart", "Germany"),
    ("Leipzig", "Germany"),
    ("Warsaw", "Poland"),
    ("Rotterdam", "Netherlands"),
    ("Amsterdam", "Netherlands"),
    ("Milan", "Italy"),
    ("Madrid", "Spain"),
    ("Paris", "France"),
    ("Istanbul", "Turkey"),
    ("Moscow", "Russia"),
    ("Saint Petersburg", "Russia"),
    ("Novosibirsk", "Russia"),
    ("Yekaterinburg", "Russia"),
    ("Guangzhou", "China"),
    ("Shanghai", "China"),
    ("Shenzhen", "China"),
    ("Xian", "China"),
    ("Tokyo", "Japan"),
    ("Seoul", "South Korea"),
    ("Mumbai", "India"),
    ("Dubai", "UAE"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_first_city_and_implied_country() {
        let p = detect_place("из гамбурга в алматы");
        assert_eq!(p.city, Some("Hamburg"));
        assert_eq!(p.country, Some("Germany"));
    }

    #[test]
    fn city_alone_implies_country() {
        let p = detect_place("доставить в алматы");
        assert_eq!(p.country, Some("Kazakhstan"));
    }

    #[test]
    fn country_alone_resolves() {
        let p = detect_place("из германии");
        assert_eq!(p.country, Some("Germany"));
        assert!(p.city.is_none());
    }

    #[test]
    fn unknown_place_is_empty() {
        assert!(detect_place("ничего тут нет").is_empty());
    }
}
