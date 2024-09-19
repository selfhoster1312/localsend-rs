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
