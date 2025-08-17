
use core::f32;
use std::collections::VecDeque;
use rand::rngs::ThreadRng;
use tracing::{info, warn};
use crate::player;
use crate::MAX_TURNS;

pub struct Game {
    pub turns: i32,
    pub players: VecDeque<player::Player>
}

impl Game {
    pub fn new() -> Self {
        Game { turns: 0, players: VecDeque::new() }
    }
pub fn get_nearest(&mut self, source: &player::Player) -> Option<(usize, &mut player::Player)> {
        let mut min_distance = f32::MAX;
        let mut target = None;
        for (idx, player) in self.players.iter_mut().enumerate() {
            if source.name != player.name {
                let distance = source.loc.distance(&source.loc);
                if distance < min_distance {
                    min_distance = distance;
                    target = Some((idx, player));
                }
            }
        }
        target
    }
    pub fn run_simulation(&mut self, rng: &mut ThreadRng) -> i32 {
        while self.players.len() > 1 {
            let mut player = self.players.pop_front().unwrap();
            let (idx, nearest_player) = self.get_nearest(&player).unwrap();
            if player.in_range(&nearest_player.loc) {
                info!("{} is in range of {}", player.name, nearest_player.name);
                if player.attack(nearest_player, rng) {
                    let damage_done = player.damage(nearest_player, rng);
                    info!("{} hit {} for {} damage", player.name, nearest_player.name, damage_done);
                    if nearest_player.is_dead() {
                        warn!("{} defeated {}", player.name, nearest_player.name);
                        drop(self.players.remove(idx));
                    }
                }
                else {
                    info!("{} missed", player.name);
                }
            } else {
                let distance = player.loc.distance(&nearest_player.loc);
                info!("{} is moving towards {} at a distance of {}", player.name, nearest_player.name, distance);
                player.move_towards(&nearest_player.loc);
            }
            self.players.push_back(player);
            self.turns += 1;
            if self.turns > MAX_TURNS {
                warn!("Battle is taking too many turns: {}", self.turns);
                break;
            }
        }
        self.turns
    }
}
