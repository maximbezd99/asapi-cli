use anyhow::{bail, Result};
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Country {
    pub code: &'static str,
    pub name: &'static str,
}

pub const COUNTRIES: &[Country] = &[
    Country {
        code: "ae",
        name: "United Arab Emirates",
    },
    Country {
        code: "ag",
        name: "Antigua and Barbuda",
    },
    Country {
        code: "ai",
        name: "Anguilla",
    },
    Country {
        code: "al",
        name: "Albania",
    },
    Country {
        code: "am",
        name: "Armenia",
    },
    Country {
        code: "ao",
        name: "Angola",
    },
    Country {
        code: "ar",
        name: "Argentina",
    },
    Country {
        code: "at",
        name: "Austria",
    },
    Country {
        code: "au",
        name: "Australia",
    },
    Country {
        code: "az",
        name: "Azerbaijan",
    },
    Country {
        code: "bb",
        name: "Barbados",
    },
    Country {
        code: "bd",
        name: "Bangladesh",
    },
    Country {
        code: "be",
        name: "Belgium",
    },
    Country {
        code: "bf",
        name: "Burkina Faso",
    },
    Country {
        code: "bg",
        name: "Bulgaria",
    },
    Country {
        code: "bh",
        name: "Bahrain",
    },
    Country {
        code: "bj",
        name: "Benin",
    },
    Country {
        code: "bm",
        name: "Bermuda",
    },
    Country {
        code: "bn",
        name: "Brunei",
    },
    Country {
        code: "bo",
        name: "Bolivia",
    },
    Country {
        code: "br",
        name: "Brazil",
    },
    Country {
        code: "bs",
        name: "Bahamas",
    },
    Country {
        code: "bt",
        name: "Bhutan",
    },
    Country {
        code: "bw",
        name: "Botswana",
    },
    Country {
        code: "by",
        name: "Belarus",
    },
    Country {
        code: "bz",
        name: "Belize",
    },
    Country {
        code: "ca",
        name: "Canada",
    },
    Country {
        code: "cd",
        name: "DR Congo",
    },
    Country {
        code: "cg",
        name: "Republic of Congo",
    },
    Country {
        code: "ch",
        name: "Switzerland",
    },
    Country {
        code: "ci",
        name: "Ivory Coast",
    },
    Country {
        code: "cl",
        name: "Chile",
    },
    Country {
        code: "cm",
        name: "Cameroon",
    },
    Country {
        code: "cn",
        name: "China",
    },
    Country {
        code: "co",
        name: "Colombia",
    },
    Country {
        code: "cr",
        name: "Costa Rica",
    },
    Country {
        code: "cv",
        name: "Cape Verde",
    },
    Country {
        code: "cy",
        name: "Cyprus",
    },
    Country {
        code: "cz",
        name: "Czech Republic",
    },
    Country {
        code: "de",
        name: "Germany",
    },
    Country {
        code: "dk",
        name: "Denmark",
    },
    Country {
        code: "dm",
        name: "Dominica",
    },
    Country {
        code: "do",
        name: "Dominican Republic",
    },
    Country {
        code: "dz",
        name: "Algeria",
    },
    Country {
        code: "ec",
        name: "Ecuador",
    },
    Country {
        code: "ee",
        name: "Estonia",
    },
    Country {
        code: "eg",
        name: "Egypt",
    },
    Country {
        code: "es",
        name: "Spain",
    },
    Country {
        code: "fi",
        name: "Finland",
    },
    Country {
        code: "fj",
        name: "Fiji",
    },
    Country {
        code: "fm",
        name: "Micronesia",
    },
    Country {
        code: "fr",
        name: "France",
    },
    Country {
        code: "ga",
        name: "Gabon",
    },
    Country {
        code: "gb",
        name: "United Kingdom",
    },
    Country {
        code: "gd",
        name: "Grenada",
    },
    Country {
        code: "ge",
        name: "Georgia",
    },
    Country {
        code: "gh",
        name: "Ghana",
    },
    Country {
        code: "gm",
        name: "Gambia",
    },
    Country {
        code: "gr",
        name: "Greece",
    },
    Country {
        code: "gt",
        name: "Guatemala",
    },
    Country {
        code: "gw",
        name: "Guinea-Bissau",
    },
    Country {
        code: "gy",
        name: "Guyana",
    },
    Country {
        code: "hk",
        name: "Hong Kong",
    },
    Country {
        code: "hn",
        name: "Honduras",
    },
    Country {
        code: "hr",
        name: "Croatia",
    },
    Country {
        code: "hu",
        name: "Hungary",
    },
    Country {
        code: "id",
        name: "Indonesia",
    },
    Country {
        code: "ie",
        name: "Ireland",
    },
    Country {
        code: "il",
        name: "Israel",
    },
    Country {
        code: "in",
        name: "India",
    },
    Country {
        code: "iq",
        name: "Iraq",
    },
    Country {
        code: "is",
        name: "Iceland",
    },
    Country {
        code: "it",
        name: "Italy",
    },
    Country {
        code: "jm",
        name: "Jamaica",
    },
    Country {
        code: "jo",
        name: "Jordan",
    },
    Country {
        code: "jp",
        name: "Japan",
    },
    Country {
        code: "ke",
        name: "Kenya",
    },
    Country {
        code: "kg",
        name: "Kyrgyzstan",
    },
    Country {
        code: "kh",
        name: "Cambodia",
    },
    Country {
        code: "kn",
        name: "Saint Kitts and Nevis",
    },
    Country {
        code: "kr",
        name: "South Korea",
    },
    Country {
        code: "kw",
        name: "Kuwait",
    },
    Country {
        code: "ky",
        name: "Cayman Islands",
    },
    Country {
        code: "kz",
        name: "Kazakhstan",
    },
    Country {
        code: "la",
        name: "Laos",
    },
    Country {
        code: "lb",
        name: "Lebanon",
    },
    Country {
        code: "lc",
        name: "Saint Lucia",
    },
    Country {
        code: "lk",
        name: "Sri Lanka",
    },
    Country {
        code: "lr",
        name: "Liberia",
    },
    Country {
        code: "lt",
        name: "Lithuania",
    },
    Country {
        code: "lu",
        name: "Luxembourg",
    },
    Country {
        code: "lv",
        name: "Latvia",
    },
    Country {
        code: "ly",
        name: "Libya",
    },
    Country {
        code: "ma",
        name: "Morocco",
    },
    Country {
        code: "md",
        name: "Moldova",
    },
    Country {
        code: "me",
        name: "Montenegro",
    },
    Country {
        code: "mg",
        name: "Madagascar",
    },
    Country {
        code: "mk",
        name: "North Macedonia",
    },
    Country {
        code: "ml",
        name: "Mali",
    },
    Country {
        code: "mm",
        name: "Myanmar",
    },
    Country {
        code: "mn",
        name: "Mongolia",
    },
    Country {
        code: "mo",
        name: "Macau",
    },
    Country {
        code: "mr",
        name: "Mauritania",
    },
    Country {
        code: "ms",
        name: "Montserrat",
    },
    Country {
        code: "mt",
        name: "Malta",
    },
    Country {
        code: "mu",
        name: "Mauritius",
    },
    Country {
        code: "mv",
        name: "Maldives",
    },
    Country {
        code: "mw",
        name: "Malawi",
    },
    Country {
        code: "mx",
        name: "Mexico",
    },
    Country {
        code: "my",
        name: "Malaysia",
    },
    Country {
        code: "mz",
        name: "Mozambique",
    },
    Country {
        code: "na",
        name: "Namibia",
    },
    Country {
        code: "ne",
        name: "Niger",
    },
    Country {
        code: "ng",
        name: "Nigeria",
    },
    Country {
        code: "ni",
        name: "Nicaragua",
    },
    Country {
        code: "nl",
        name: "Netherlands",
    },
    Country {
        code: "no",
        name: "Norway",
    },
    Country {
        code: "np",
        name: "Nepal",
    },
    Country {
        code: "nz",
        name: "New Zealand",
    },
    Country {
        code: "om",
        name: "Oman",
    },
    Country {
        code: "pa",
        name: "Panama",
    },
    Country {
        code: "pe",
        name: "Peru",
    },
    Country {
        code: "pg",
        name: "Papua New Guinea",
    },
    Country {
        code: "ph",
        name: "Philippines",
    },
    Country {
        code: "pk",
        name: "Pakistan",
    },
    Country {
        code: "pl",
        name: "Poland",
    },
    Country {
        code: "pt",
        name: "Portugal",
    },
    Country {
        code: "pw",
        name: "Palau",
    },
    Country {
        code: "py",
        name: "Paraguay",
    },
    Country {
        code: "qa",
        name: "Qatar",
    },
    Country {
        code: "ro",
        name: "Romania",
    },
    Country {
        code: "rs",
        name: "Serbia",
    },
    Country {
        code: "ru",
        name: "Russia",
    },
    Country {
        code: "rw",
        name: "Rwanda",
    },
    Country {
        code: "sa",
        name: "Saudi Arabia",
    },
    Country {
        code: "sb",
        name: "Solomon Islands",
    },
    Country {
        code: "sc",
        name: "Seychelles",
    },
    Country {
        code: "se",
        name: "Sweden",
    },
    Country {
        code: "sg",
        name: "Singapore",
    },
    Country {
        code: "si",
        name: "Slovenia",
    },
    Country {
        code: "sk",
        name: "Slovakia",
    },
    Country {
        code: "sl",
        name: "Sierra Leone",
    },
    Country {
        code: "sn",
        name: "Senegal",
    },
    Country {
        code: "sr",
        name: "Suriname",
    },
    Country {
        code: "st",
        name: "Sao Tome and Principe",
    },
    Country {
        code: "sv",
        name: "El Salvador",
    },
    Country {
        code: "sz",
        name: "Eswatini",
    },
    Country {
        code: "tc",
        name: "Turks and Caicos",
    },
    Country {
        code: "td",
        name: "Chad",
    },
    Country {
        code: "th",
        name: "Thailand",
    },
    Country {
        code: "tj",
        name: "Tajikistan",
    },
    Country {
        code: "tm",
        name: "Turkmenistan",
    },
    Country {
        code: "tn",
        name: "Tunisia",
    },
    Country {
        code: "to",
        name: "Tonga",
    },
    Country {
        code: "tr",
        name: "Turkey",
    },
    Country {
        code: "tt",
        name: "Trinidad and Tobago",
    },
    Country {
        code: "tw",
        name: "Taiwan",
    },
    Country {
        code: "tz",
        name: "Tanzania",
    },
    Country {
        code: "ua",
        name: "Ukraine",
    },
    Country {
        code: "ug",
        name: "Uganda",
    },
    Country {
        code: "us",
        name: "United States",
    },
    Country {
        code: "uy",
        name: "Uruguay",
    },
    Country {
        code: "uz",
        name: "Uzbekistan",
    },
    Country {
        code: "vc",
        name: "Saint Vincent and the Grenadines",
    },
    Country {
        code: "ve",
        name: "Venezuela",
    },
    Country {
        code: "vg",
        name: "British Virgin Islands",
    },
    Country {
        code: "vn",
        name: "Vietnam",
    },
    Country {
        code: "ye",
        name: "Yemen",
    },
    Country {
        code: "za",
        name: "South Africa",
    },
    Country {
        code: "zw",
        name: "Zimbabwe",
    },
];

pub fn validate_country(value: &str) -> Result<String> {
    let code = value.trim().to_ascii_lowercase();
    if !COUNTRIES.iter().any(|country| country.code == code) {
        bail!("unsupported App Store country code '{value}'; run 'asapi list countries'");
    }
    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_and_normalizes_country() {
        assert_eq!(validate_country(" US ").unwrap(), "us");
        assert!(validate_country("xx").is_err());
    }
}
