use rand::rngs::ThreadRng;
use tracing::{error, info};
use clap::Parser;
mod dice;
mod game;
mod names;
mod player;
mod serialization;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_TURNS: i32 = 256;
const MAX_PLAYERS: u8 = 64;

#[derive(Parser, Debug)]
#[command(name = "battlers")]
#[command(version = "2.0.0")]
#[command(about = "Simulation of a skirmish", long_about = None)]
struct Args {
    /// Number of random players to simulate
    #[arg(short, long, default_value_t = 2)]
    players: u8,
    /// Path to simulation configuration YAML file
    #[arg(short, long)]
    config: Option<String>,
}

fn main() {
    // turn on loggging
    tracing_subscriber::fmt::init();
    info!("battlers/{}", VERSION);

    // get the command arguments
    let args = Args::parse();

    // initialize the random number generator
    let mut rng: ThreadRng = rand::rng();

    // create a new game engine and add players
    let mut game = game::Game::new();
    
    match args.config {
        Some(config_path) => {
            // Load players from YAML configuration
            match serialization::load_simulation_config(&config_path) {
                Ok(config) => {
                    let players = serialization::players_from_config(config);
                    for player in players {
                        game.players.push_back(player);
                    }
                }
                Err(e) => {
                    error!("Failed to load configuration from {}: {}", config_path, e);
                    return;
                }
            }
        }
        None => {
            // Generate random players
            if args.players > MAX_PLAYERS {
                error!("too many players requested, {} is the max", MAX_PLAYERS);
                return;
            }
            
            for _ in 0..args.players {
                let mut player = player::Player::new(names::get_random_name(&mut rng));
                player.randomize(&mut rng);
                info!("{:?}", player);
                game.players.push_back(player);
            }
        }
    }
    
    info!("{} players enter the skirmish", game.players.len());

    // run the simulation with the players
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
