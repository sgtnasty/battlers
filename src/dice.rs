use rand::{Rng, rngs::ThreadRng};
use tracing::debug;

pub fn roll3d6(rng: &mut ThreadRng) -> i32 {
    let roll1 = rng.random_range(1..=6);
    debug!("rolled {}/6", roll1);
    let roll2 = rng.random_range(1..=6);
    debug!("rolled {}/6", roll2);
    let roll3 = rng.random_range(1..=6);
    debug!("rolled {}/6", roll3);
    roll1 + roll2 + roll3
}

pub fn roll1d20(rng: &mut ThreadRng) -> i32 {
    let roll = rng.random_range(1..=20);
    debug!("rolled {}/20", roll);
    roll
}

pub fn roll1d8(rng: &mut ThreadRng) -> i32 {
    let roll = rng.random_range(1..=8);
    debug!("rolled {}/8", roll);
    roll
}
