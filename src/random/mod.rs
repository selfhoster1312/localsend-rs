use pure_rng::PureRng;
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
    let mut rng = PureRng::new(seed);
    format!(
        "{} {}",
        ADJECTIVES.choose(&mut rng).unwrap(),
        FRUITS.choose(&mut rng).unwrap(),
    )
}

#[cfg(test)]
mod tests {
    use super::alias_from_seed;

    #[test]
    fn check_determinism() {
        assert_eq!("Energetic Cherry", &alias_from_seed("ACAB"));
    }
}
