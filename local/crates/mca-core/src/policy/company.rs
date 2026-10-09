/// Company-owned facts. Everything the agent is allowed to state as fact lives
/// here, so a model prompt can never become the source of a company claim.
pub const COMPANY_NAME: &str = "MCA Logistics";

/// Canonical English country code → Russian display name. Only countries the
/// company actually serves (or that appear in the lexicon) need an entry; the
/// fallback returns the input unchanged so unknown countries still render.
pub fn country_ru(canonical: &str) -> String {
    match canonical {
        "Germany" => "Германия",
        "Kazakhstan" => "Казахстан",
        "Russia" => "Россия",
        "China" => "Китай",
        "Turkey" => "Турция",
        "UAE" => "ОАЭ",
        "Poland" => "Польша",
        "Italy" => "Италия",
        "Spain" => "Испания",
        "France" => "Франция",
        "Netherlands" => "Нидерланды",
        "Lithuania" => "Литва",
        "Latvia" => "Латвия",
        "Estonia" => "Эстония",
        "USA" => "США",
        "India" => "Индия",
        "Japan" => "Япония",
        "South Korea" => "Южная Корея",
        "UK" => "Великобритания",
        "Ukraine" => "Украина",
        "Belarus" => "Беларусь",
        "Uzbekistan" => "Узбекистан",
        "Kyrgyzstan" => "Кыргызстан",
        "Azerbaijan" => "Азербайджан",
        "Georgia" => "Грузия",
        "Armenia" => "Армения",
        "Czech Republic" => "Чехия",
        other => other,
    }
    .to_string()
}

/// Thousands-separated weight, or a friendly placeholder.
pub fn format_weight(kg: Option<f64>) -> String {
    match kg {
        Some(v) if v >= 1000.0 => {
            let tons = v / 1000.0;
            let rounded = (tons * 100.0).round() / 100.0;
            let with_sep = thousands(rounded);
            format!("{with_sep} т ({} кг)", thousands(v.round()))
        }
        Some(v) => format!("{} кг", thousands(v.round())),
        None => "Не указан".to_string(),
    }
}

/// Volume in m³, or a friendly placeholder.
pub fn format_volume(m3: Option<f64>) -> String {
    match m3 {
        Some(v) => format!("{} м³", format_decimal(v)),
        None => "Не указан".to_string(),
    }
}

fn thousands(value: f64) -> String {
    let rounded = value.round() as i64;
    let digits = rounded.abs().to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(' ');
        }
        out.push(ch);
    }
    if rounded < 0 {
        format!("-{out}")
    } else {
        out
    }
}

fn format_decimal(value: f64) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    let s = format!("{rounded}");
    s
}

#[cfg(test)]
mod tests;
