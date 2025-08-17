use rand::{Rng, rngs::ThreadRng};
use tracing::{error, info};
mod dice;
mod game;
mod names;
mod player;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_TURNS: i32 = 256;

fn main() {
    // turn on loggging
    tracing_subscriber::fmt::init();
    info!("battlers/{}", VERSION);

    // initialize the random number generator
    let mut rng: ThreadRng = rand::rng();

    let mut game = game::Game::new();
    for _ in 0..rng.random_range(2..10) {
        let mut player = player::Player::new(names::get_random_name(&mut rng));
        player.randomize(&mut rng);
        info!("{:?}", player);
        game.players.push_back(player);
    }
    info!("{} players enter the skirmish", game.players.len());
    let turns_elapsed = game.run_simulation(&mut rng);
    if game.players.len() == 1 {
        info!(
            "{} is the winner in {} turns with {} of {} hits left",
            game.players[0].name,
            turns_elapsed,
            game.players[0].armor.curr,
            game.players[0].armor.base
        );
    } else {
        error!("inconclusive results")
    }
}
