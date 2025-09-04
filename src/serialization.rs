use serde::Deserialize;
use std::fs;
use std::path::Path;
use tracing::{error, info};
use crate::player::{Player, Location};

#[derive(Deserialize, Debug)]
pub struct LocationConfig {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Deserialize, Debug)]
pub struct PlayerConfig {
    pub name: String,
    pub attack: i32,
    pub defense: i32,
    pub armor: i32,
    pub power: i32,
    pub speed: i32,
    pub range: i32,
    pub loc: LocationConfig,
}

#[derive(Deserialize, Debug)]
pub struct SimulationConfig {
    pub players: Vec<PlayerConfig>,
}

impl From<LocationConfig> for Location {
    fn from(config: LocationConfig) -> Self {
        Location::new(config.x, config.y, config.z)
    }
}

impl From<&LocationConfig> for Location {
    fn from(config: &LocationConfig) -> Self {
        Location::new(config.x, config.y, config.z)
    }
}

impl From<PlayerConfig> for Player {
    fn from(config: PlayerConfig) -> Self {
        let mut player = Player::new(&config.name);
        
        // Set attributes with base and current values
        player.attack.set(config.attack);
        player.defense.set(config.defense);
        player.armor.set(config.armor);
        player.power.set(config.power);
        player.speed.set(config.speed);
        player.range.set(config.range);
        
        // Set location
        player.loc = Location::from(&config.loc);
        
        player
    }
}

pub fn load_simulation_config<P: AsRef<Path>>(path: P) -> Result<SimulationConfig, Box<dyn std::error::Error>> {
    let path = path.as_ref();
    info!("Loading simulation configuration from: {}", path.display());
    
    let content = fs::read_to_string(path)
        .map_err(|e| {
            error!("Failed to read file {}: {}", path.display(), e);
            e
        })?;
    
    let config: SimulationConfig = serde_yaml::from_str(&content)
        .map_err(|e| {
            error!("Failed to parse YAML from {}: {}", path.display(), e);
            e
        })?;
    
    info!("Successfully loaded {} players from configuration", config.players.len());
    Ok(config)
}

pub fn players_from_config(config: SimulationConfig) -> Vec<Player> {
    config.players
        .into_iter()
        .map(|player_config| {
            let player = Player::from(player_config);
            info!("{:?}", player);
            player
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_location_conversion() {
        let loc_config = LocationConfig { x: 1.0, y: 2.0, z: 3.0 };
        let location: Location = loc_config.into();
        
        assert_eq!(location.x, 1.0);
        assert_eq!(location.y, 2.0);
        assert_eq!(location.z, 3.0);
    }
    
    #[test]
    fn test_player_conversion() {
        let player_config = PlayerConfig {
            name: "Test Player".to_string(),
            attack: 10,
            defense: 12,
            armor: 14,
            power: 8,
            speed: 16,
            range: 6,
            loc: LocationConfig { x: 5.0, y: 10.0, z: 0.0 },
        };
        
        let player: Player = player_config.into();
        
        assert_eq!(player.name, "Test Player");
        assert_eq!(player.attack.base, 10);
        assert_eq!(player.defense.base, 12);
        assert_eq!(player.armor.base, 14);
        assert_eq!(player.power.base, 8);
        assert_eq!(player.speed.base, 16);
        assert_eq!(player.range.base, 6);
        assert_eq!(player.loc.x, 5.0);
        assert_eq!(player.loc.y, 10.0);
        assert_eq!(player.loc.z, 0.0);
    }
}
