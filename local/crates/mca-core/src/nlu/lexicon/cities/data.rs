//! Static city rows. The tuples are `(canonical_name, russian_label,
//! surface_forms)`; the canonical English name is what lands in the structured
//! application, so downstream reports stay locale-independent.

pub const CITIES: &[(&str, &str, &[&str])] = &[
    (
        "Almaty",
        "Алматы",
        &["алматы", "алма аты", "алма-аты", "almaty"],
    ),
    (
        "Astana",
        "Астана",
        &["астан", "нур-султан", "astana", "nur-sultan"],
    ),
    (
        "Shymkent",
        "Шымкент",
        &["шикемент", "чымкент", "shymkent", "shimkent"],
    ),
    ("Karaganda", "Караганда", &["караганда", "karaganda"]),
    ("Hamburg", "Гамбург", &["гамбург", "гамбурга", "hamburg"]),
    ("Berlin", "Берлин", &["берлин", "berlin"]),
    (
        "Munich",
        "Мюнхен",
        &["мюнхен", "мунхен", "munich", "munchen"],
    ),
    ("Frankfurt", "Франкфурт", &["франкфурт", "frankfurt"]),
    ("Cologne", "Кёльн", &["кельн", "кёльн", "cologne", "koln"]),
    ("Stuttgart", "Штутгарт", &["штутгарт", "stuttgart"]),
    ("Leipzig", "Лейпциг", &["лейпциг", "leipzig"]),
    ("Warsaw", "Варшава", &["варшава", "warsaw", "warszawa"]),
    ("Rotterdam", "Роттердам", &["роттердам", "rotterdam"]),
    ("Amsterdam", "Амстердам", &["амстердам", "amsterdam"]),
    ("Milan", "Милан", &["милан", "milan", "milano"]),
    ("Madrid", "Мадрид", &["мадрид", "madrid"]),
    ("Paris", "Париж", &["париж", "paris"]),
    ("Istanbul", "Стамбул", &["стамбул", "istanbul"]),
    (
        "Moscow",
        "Москва",
        &["москва", "москву", "moscow", "moskva"],
    ),
    (
        "Saint Petersburg",
        "Санкт-Петербург",
        &[
            "санкт петербург",
            "петербург",
            "спб",
            "petersburg",
            "st petersburg",
        ],
    ),
    (
        "Novosibirsk",
        "Новосибирск",
        &["новосибирск", "novosibirsk"],
    ),
    (
        "Yekaterinburg",
        "Екатеринбург",
        &["екатеринбург", "ekaterinburg"],
    ),
    (
        "Guangzhou",
        "Гуанчжоу",
        &["гуанчжоу", "кантон", "guangzhou"],
    ),
    ("Shanghai", "Шанхай", &["шанхай", "shanghai"]),
    ("Shenzhen", "Шэньчжэнь", &["шэньчжэнь", "shenzhen"]),
    ("Xian", "Сиань", &["сиань", "xian"]),
    ("Tokyo", "Токио", &["токио", "tokyo"]),
    ("Seoul", "Сеул", &["сеул", "seoul"]),
    ("Mumbai", "Мумбаи", &["мумбаи", "mumbai", "дели", "delhi"]),
    ("Dubai", "Дубай", &["дубай", "dubai"]),
];
