use rand::rngs::ThreadRng;
use tracing::{error, info};
use clap::Parser;
mod dice;
mod game;
mod names;
mod player;
mod serialization;
mod app;
mod tui;

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
    /// Enable TUI mode for interactive battle visualization
    #[arg(short, long)]
    tui: bool,
}

fn main() {
    // get the command arguments
    let args = Args::parse();
    
    if args.tui {
        // Run in TUI mode
        run_tui_mode(args);
    } else {
        // Run in CLI mode
        run_cli_mode(args);
    }
}

fn run_tui_mode(args: Args) {
    // Initialize terminal
    let terminal = match tui::setup_terminal() {
        Ok(terminal) => terminal,
        Err(e) => {
            eprintln!("Failed to setup terminal: {}", e);
            return;
        }
    };
    
    // Create app and load players
    let mut app = app::App::new();
    let players = load_players(args);
    app.add_players(players);
    
    // Create TUI and run
    let mut tui_instance = tui::Tui::new(terminal);
    if let Err(e) = tui_instance.run(app) {
        eprintln!("TUI error: {}", e);
    }
    
    // Restore terminal
    if let Err(e) = tui::restore_terminal() {
        eprintln!("Failed to restore terminal: {}", e);
    }
}

fn run_cli_mode(args: Args) {
    // turn on logging
    tracing_subscriber::fmt::init();
    info!("battlers/{}", VERSION);

    // initialize the random number generator
    let mut rng: ThreadRng = rand::rng();

    // create a new game engine and add players
    let mut game = game::Game::new();
    let players = load_players(args);
    
    for player in players {
        info!("{:?}", player);
        game.players.push_back(player);
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

fn load_players(args: Args) -> Vec<player::Player> {
    let mut rng: ThreadRng = rand::rng();
    
    match args.config {
        Some(config_path) => {
            // Load players from YAML configuration
            match serialization::load_simulation_config(&config_path) {
                Ok(config) => serialization::players_from_config(config),
                Err(e) => {
                    error!("Failed to load configuration from {}: {}", config_path, e);
                    Vec::new()
                }
            }
        }
        None => {
            // Generate random players
            if args.players > MAX_PLAYERS {
                error!("too many players requested, {} is the max", MAX_PLAYERS);
                return Vec::new();
            }
            
            let mut players = Vec::new();
            for _ in 0..args.players {
                let mut player = player::Player::new(names::get_random_name(&mut rng));
                player.randomize(&mut rng);
                players.push(player);
            }
            players
        }
    }
}
