//! Russian Place Name Generator
//! Generates realistic Russian-style settlement names

use rand_chacha::ChaCha8Rng;
use rand::{Rng, SeedableRng};

// Name components for generating Russian place names
const PREFIXES: &[&str] = &[
    "Ново", "Старо", "Красно", "Бело", "Верхне", "Нижне",
    "Больше", "Мало", "Средне", "Черно", "Свято", "Дальне",
    "Западно", "Восточно", "Северо", "Южно", "При", "За",
];

const ROOTS: &[&str] = &[
    "горск", "речинск", "озёрск", "сибирск", "таёжн",
    "ельцов", "берёзов", "сосновк", "кедров", "ольхов",
    "камен", "песчан", "глинян", "торфян", "болотн",
    "лесн", "полев", "степн", "морск", "волжск",
    "донск", "уральск", "алтайск", "амурск", "ленск",
];

const SUFFIXES: &[&str] = &[
    "ое", "ск", "ка", "ово", "ино", "ево", "цы", "и",
    "град", "поль", "дар", "горье", "речье", "бор",
];

/// Generate a Russian-style place name from seed
pub fn generate_name(seed: u64) -> String {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    // Decide name pattern: prefix+root+suffix or just root+suffix
    let use_prefix = rng.gen_bool(0.7); // 70% chance of prefix

    let root = ROOTS[rng.gen_range(0..ROOTS.len())];
    let suffix = SUFFIXES[rng.gen_range(0..SUFFIXES.len())];

    if use_prefix {
        let prefix = PREFIXES[rng.gen_range(0..PREFIXES.len())];
        format!("{}{}{}", prefix, root, suffix)
    } else {
        format!("{}{}", root, suffix)
    }
}

/// Generate a name with specific characteristics (for debugging/testing)
pub fn generate_name_variants(seed: u64, count: usize) -> Vec<String> {
    let _rng = ChaCha8Rng::seed_from_u64(seed);
    let mut names = Vec::with_capacity(count);

    for i in 0..count {
        let variant_seed = seed.wrapping_add(i as u64);
        names.push(generate_name(variant_seed));
    }

    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_generation() {
        let name = generate_name(12345);
        assert!(!name.is_empty());
        assert!(name.len() > 3);
    }

    #[test]
    fn test_name_variants() {
        let names = generate_name_variants(12345, 10);
        assert_eq!(names.len(), 10);
        // Names should be deterministic
        let names2 = generate_name_variants(12345, 10);
        assert_eq!(names, names2);
    }
}
