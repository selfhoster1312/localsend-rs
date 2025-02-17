use rand::seq::SliceRandom;
use rand::thread_rng;

mod words;
pub use words::{ADJECTIVES, FRUITS};

pub fn random_alias() -> String {
    let mut rng = thread_rng();
    format!(
        "{} {}",
        ADJECTIVES.choose(&mut rng).unwrap(),
        FRUITS.choose(&mut rng).unwrap(),
    )
}

pub fn alias_from_seed(seed: &str) -> String {
    let (int1, int2) = pseudorandom_ints_from_seed(seed);

    let adjectives_index: usize = int1 as usize % ADJECTIVES.len();
    let fruits_index: usize = int2 as usize % FRUITS.len();

    format!("{} {}", ADJECTIVES[adjectives_index], FRUITS[fruits_index],)
}

/// Generate two pseudo-random integers from a stringy seed.
///
/// These integers can be overflown and are guaranteed to not be truly random. In fact,
/// it is guaranteed to have a determinsitic output depending on the seed.
///
/// Do not use this function for security-related purposes.
///
/// This function is for example used to derive a stable alias from the TLS fingerprint
/// of the localsend client. It's supposed to be called with a fixed-length seed,
/// but does support variable-length seeds as well.
pub fn pseudorandom_ints_from_seed(seed: &str) -> (u16, u16) {
    if seed.len() == 0 {
        return (0, 0);
    }

    let first_half = seed[0..seed.len() / 2]
        .as_bytes()
        .iter()
        .fold(0, |acc: u16, x| {
            let (acc, _) = acc.overflowing_add(*x as u16);
            acc
        });

    let second_half =
        seed[seed.len() / 2..seed.len() - 1]
            .as_bytes()
            .iter()
            .fold(0, |acc: u16, x| {
                let (acc, _) = acc.overflowing_add(*x as u16);
                acc
            });

    (first_half, second_half)
}

#[cfg(test)]
mod tests {
    use super::alias_from_seed;

    #[test]
    fn check_determinism() {
        assert_eq!("Great Melon", &alias_from_seed("ACAB"));
    }
}
