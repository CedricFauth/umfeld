//! Display cleanup for fuel station names and places.

use unicode_normalization::UnicodeNormalization;

/// Longest first, so " GmbH & Co. KG" isn't cut down to " GmbH & Co.".
const LEGAL_FORMS: [&str; 5] = [" GmbH & Co. KG", " GmbH", " AG", " KG", " e.K."];

/// `brand` without legal-form suffixes, or `name` the same way if `brand` is empty.
/// Falls back to the raw `name` if nothing is left.
pub fn display_name(brand: &str, name: &str) -> String {
    let source = if brand.trim().is_empty() { name } else { brand };
    let source = nfc(source);
    let cleaned = strip_legal_forms(&source);
    if cleaned.is_empty() {
        nfc(name)
    } else {
        cleaned.to_owned()
    }
}

/// Title case if `place` is all caps, else unchanged.
pub fn place(place: &str) -> String {
    let place = nfc(place);
    // ß has no capital in everyday use, so "GROßENHEIDORN" still counts as all caps
    let all_caps = place.chars().any(char::is_alphabetic)
        && !place.chars().any(|c| c.is_lowercase() && c != 'ß');
    if all_caps { title_case(&place) } else { place }
}

fn nfc(s: &str) -> String {
    s.nfc().collect()
}

fn strip_legal_forms(s: &str) -> &str {
    let mut s = s.trim();
    while let Some(rest) = LEGAL_FORMS.iter().find_map(|form| s.strip_suffix(form)) {
        s = rest.trim_end();
    }
    s
}

/// Words are separated by space or hyphen.
fn title_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut at_word_start = true;
    for c in s.chars() {
        match c {
            ' ' | '-' => {
                at_word_start = true;
                out.push(c);
            }
            'ß' => {
                at_word_start = false;
                out.push(c);
            }
            c if at_word_start && c.is_alphabetic() => {
                at_word_start = false;
                out.extend(c.to_uppercase());
            }
            c => out.extend(c.to_lowercase()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_examples() {
        let cases = [
            (
                "OTTO HATTENDORF GmbH",
                "OTTO HATTENDORF GmbH",
                "Lindhorst",
                "OTTO HATTENDORF",
                "Lindhorst",
            ),
            ("ESSO", "Esso Tankstelle", "RODENBERG", "ESSO", "Rodenberg"),
            ("ESSO", "Esso Tankstelle", "LAUENAU", "ESSO", "Lauenau"),
            (
                "Rakelbusch",
                "Rakelbusch GmbH",
                "Barsinghausen",
                "Rakelbusch",
                "Barsinghausen",
            ),
            (
                "",
                "Rakelbusch GmbH",
                "BAD MÜNDER",
                "Rakelbusch",
                "Bad Münder",
            ),
        ];
        for (brand, name, upstream_place, want_name, want_place) in cases {
            assert_eq!(display_name(brand, name), want_name);
            assert_eq!(place(upstream_place), want_place);
        }
    }

    #[test]
    fn legal_forms_are_stripped_repeatedly() {
        assert_eq!(display_name("Meyer GmbH & Co. KG", ""), "Meyer");
        assert_eq!(display_name("Meyer KG GmbH ", ""), "Meyer");
        assert_eq!(display_name("Huber e.K.", ""), "Huber");
        assert_eq!(display_name("STAR AG", ""), "STAR");
        assert_eq!(display_name("   ", "Tankstelle Süd AG"), "Tankstelle Süd");
    }

    #[test]
    fn display_name_is_never_empty() {
        assert_eq!(display_name("", "GmbH"), "GmbH");
        assert_eq!(display_name("GmbH", "Tankstelle"), "GmbH");
    }

    #[test]
    fn title_case_rules() {
        assert_eq!(place("GROßENHEIDORN"), "Großenheidorn");
        assert_eq!(place("NEUSTADT AM RÜBENBERGE"), "Neustadt Am Rübenberge");
        assert_eq!(place("HANN.-MÜNDEN"), "Hann.-Münden");
        assert_eq!(place("ÜBERLINGEN"), "Überlingen");
        assert_eq!(place("Neustadt a. Rbge."), "Neustadt a. Rbge.");
        assert_eq!(place("12345"), "12345");
    }

    #[test]
    fn output_is_nfc() {
        let decomposed = "BAD MU\u{308}NDER";
        assert_eq!(place(decomposed), "Bad M\u{fc}nder");
        assert_eq!(display_name("Mu\u{308}ller", ""), "M\u{fc}ller");
    }
}
