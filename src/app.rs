// Citation for use of AI tools:
// Date: 2025-09-05
// Prompts: "Can you integrate a TUI using the ratatui crate for rust?"
// AI Source URL: https://www.anthropic.com/claude/sonnet

use std::collections::VecDeque;
use crate::game::Game;
use crate::player::Player;

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Setup,      // Initial setup screen
    Running,    // Battle simulation is running
    Paused,     // Battle simulation is paused
    Finished,   // Battle has ended
    Quit,       // User wants to quit
}

#[derive(Debug, Clone)]
pub struct BattleEvent {
    pub turn: i32,
    pub message: String,
    pub event_type: BattleEventType,
}

#[derive(Debug, Clone)]
pub enum BattleEventType {
    Movement,
    Attack,
    Hit,
    Miss,
    Death,
    Info,
}

pub struct App {
    pub state: AppState,
    pub game: Game,
    pub battle_log: VecDeque<BattleEvent>,
    pub current_turn: i32,
    pub auto_advance: bool,
    pub tick_rate: u64, // milliseconds
    pub max_log_entries: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::Setup,
            game: Game::new(),
            battle_log: VecDeque::new(),
            current_turn: 0,
            auto_advance: false,
            tick_rate: 500, // 500ms between auto-advances
            max_log_entries: 50,
        }
    }

    pub fn add_players(&mut self, players: Vec<Player>) {
        for player in players {
            self.game.players.push_back(player);
        }
    }

    pub fn start_battle(&mut self) {
        if !self.game.players.is_empty() {
            self.state = AppState::Running;
            self.add_battle_event(
                "Battle begins!".to_string(),
                BattleEventType::Info,
            );
        }
    }

    pub fn pause_battle(&mut self) {
        if self.state == AppState::Running {
            self.state = AppState::Paused;
        }
    }

    pub fn resume_battle(&mut self) {
        if self.state == AppState::Paused {
            self.state = AppState::Running;
        }
    }

    pub fn toggle_pause(&mut self) {
        match self.state {
            AppState::Running => self.pause_battle(),
            AppState::Paused => self.resume_battle(),
            _ => {}
        }
    }

    pub fn quit(&mut self) {
        self.state = AppState::Quit;
    }

    pub fn step_battle(&mut self, rng: &mut rand::rngs::ThreadRng) -> bool {
        if self.game.players.len() <= 1 {
            self.finish_battle();
            return false;
        }

        if self.current_turn >= crate::MAX_TURNS {
            self.add_battle_event(
                format!("Battle reached maximum turns: {}", crate::MAX_TURNS),
                BattleEventType::Info,
            );
            self.finish_battle();
            return false;
        }

        // Execute one turn of the battle
        if let Some(mut player) = self.game.players.pop_front() {
            let player_name = player.name.clone();
            
            // Get target information without borrowing self
            let target_info = if let Some((target_idx, nearest_player)) = self.game.get_nearest(&player) {
                Some((target_idx, nearest_player.name.clone(), nearest_player.loc.clone()))
            } else {
                None
            };
            
            if let Some((target_idx, target_name, target_loc)) = target_info {
                if player.in_range(&target_loc) {
                    self.add_battle_event(
                        format!("{} is in range of {}", player_name, target_name),
                        BattleEventType::Info,
                    );
                    
                    // Get mutable reference to target for combat
                    let combat_result = if let Some((_, nearest_player)) = self.game.get_nearest(&player) {
                        if player.attack(nearest_player, rng) {
                            let damage_done = player.damage(nearest_player, rng);
                            let target_is_dead = nearest_player.is_dead();
                            Some((true, damage_done, target_is_dead))
                        } else {
                            Some((false, 0, false))
                        }
                    } else {
                        None
                    };
                    
                    if let Some((hit, damage_done, target_is_dead)) = combat_result {
                        if hit {
                            self.add_battle_event(
                                format!("{} hit {} for {} damage", player_name, target_name, damage_done),
                                BattleEventType::Hit,
                            );
                            
                            if target_is_dead {
                                self.add_battle_event(
                                    format!("{} defeated {}", player_name, target_name),
                                    BattleEventType::Death,
                                );
                                self.game.players.remove(target_idx);
                            }
                        } else {
                            self.add_battle_event(
                                format!("{} missed", player_name),
                                BattleEventType::Miss,
                            );
                        }
                    }
                } else {
                    let distance = player.loc.distance(&target_loc);
                    self.add_battle_event(
                        format!("{} moves towards {} (distance: {:.1})", player_name, target_name, distance),
                        BattleEventType::Movement,
                    );
                    player.move_towards(&target_loc);
                }
            }
            self.game.players.push_back(player);
        }

        self.current_turn += 1;
        self.game.turns = self.current_turn;

        // Check if battle is over
        if self.game.players.len() <= 1 {
            self.finish_battle();
            return false;
        }

        true
    }

    pub fn finish_battle(&mut self) {
        self.state = AppState::Finished;
        if self.game.players.len() == 1 {
            let winner = &self.game.players[0];
            self.add_battle_event(
                format!("{} is the winner with {}/{} health remaining!", 
                       winner.name, winner.armor.curr, winner.armor.base),
                BattleEventType::Info,
            );
        } else {
            self.add_battle_event(
                "Battle ended inconclusively".to_string(),
                BattleEventType::Info,
            );
        }
    }

    pub fn add_battle_event(&mut self, message: String, event_type: BattleEventType) {
        let event = BattleEvent {
            turn: self.current_turn,
            message,
            event_type,
        };
        
        self.battle_log.push_back(event);
        
        // Keep only the most recent entries
        while self.battle_log.len() > self.max_log_entries {
            self.battle_log.pop_front();
        }
    }

    pub fn get_winner(&self) -> Option<&Player> {
        if self.game.players.len() == 1 {
            self.game.players.front()
        } else {
            None
        }
    }

    pub fn get_battle_log(&self) -> &VecDeque<BattleEvent> {
        &self.battle_log
    }

    pub fn toggle_auto_advance(&mut self) {
        self.auto_advance = !self.auto_advance;
    }

    pub fn should_quit(&self) -> bool {
        self.state == AppState::Quit
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
