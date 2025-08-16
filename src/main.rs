
use core::f32;

use rand::{rngs::ThreadRng, Rng};
use tracing::{info, debug, error, warn, trace};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const MAXTURNS: i32 = 256;

pub fn roll3d6(rng: &mut ThreadRng) -> i32 {
    let roll1 = rng.random_range(1..=6);
    let roll2 = rng.random_range(1..=6);
    let roll3 = rng.random_range(1..=6);
    roll1 + roll2 + roll3
}

pub fn roll1d20(rng: &mut ThreadRng) -> i32 {
    let roll = rng.random_range(1..=20);
    roll
}

pub fn roll1d8(rng: &mut ThreadRng) -> i32 {
    let roll = rng.random_range(1..=8);
    roll
}

#[derive(Debug)]
pub enum Attribute {
    Attack,
    Defense,
    Armor,
    Power,
    Speed,
    Range
}

#[derive(Debug)]
pub struct PlayerAttribute {
    pub name: Attribute,
    pub base: i32,
    pub curr: i32,
}

impl PlayerAttribute {
    pub fn new(name: Attribute) -> Self {
        PlayerAttribute { name: name, base: 0, curr: 0 }
    }
    pub fn set(&mut self, value: i32) {
        self.base = value;
        self.curr = self.base;
    }
    pub fn bonus(&self) -> i32 {
        let bv: f32 = (self.curr as f32 - 10.5) / 2.0;
        return bv as i32;
    }
    pub fn randomize(&mut self, rng: &mut ThreadRng) {
        self.base = roll3d6(rng);
        self.curr = self.base;
    }
}

#[derive(Debug, Clone)]
pub struct Location {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Location {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Location { x: x, y: y, z: z }
    }
    pub fn distance(&self, target: &Location) -> f32 {
        let dx = self.x - target.x;
        let dy = self.y - target.y;
        let dz = self.z - target.z;
        let pdx = dx.powf(2.0);
        let pdy = dy.powf(2.0);
        let pdz = dz.powf(2.0);
        let i = pdx + pdy + pdz;
        i.sqrt()
    }
    pub fn randomize(&mut self, rng: &mut ThreadRng) {
        self.x = roll3d6(rng) as f32;
        self.y = roll3d6(rng) as f32;
        self.z = 0.0;
    }
}

#[derive(Debug)]
pub struct Player {
    pub name: String,
    pub attack: PlayerAttribute,
    pub defense: PlayerAttribute,
    pub armor: PlayerAttribute,
    pub power: PlayerAttribute,
    pub speed: PlayerAttribute,
    pub range: PlayerAttribute,
    pub loc: Location
}

impl Player {
    pub fn new(name: &str) -> Self {
        Player { 
            name: String::from(name),
            attack: PlayerAttribute::new(Attribute::Attack),
            defense: PlayerAttribute::new(Attribute::Defense), 
            armor: PlayerAttribute::new(Attribute::Armor), 
            power: PlayerAttribute::new(Attribute::Power), 
            speed: PlayerAttribute::new(Attribute::Speed), 
            range: PlayerAttribute::new(Attribute::Range), 
            loc: Location::new(0.0, 0.0, 0.0)
        }
    }
    pub fn randomize(&mut self, rng: &mut ThreadRng) {
        self.attack.randomize(rng);
        self.defense.randomize(rng);
        self.armor.randomize(rng);
        self.power.randomize(rng);
        self.speed.randomize(rng);
        self.range.randomize(rng);
        self.loc.randomize(rng);
    }
    pub fn move_towards(&mut self, target: &Location) {
        let distance = self.loc.distance(target);
        if distance <= self.speed.curr as f32 {
            return;
        }
        let dx_normalized = (target.x - self.loc.x) / distance;
        let dy_normalized = (target.y - self.loc.y) / distance;
        let new_x = self.loc.x + dx_normalized * self.speed.curr as f32;
        let new_y = self.loc.y + dy_normalized * self.speed.curr as f32;
        self.loc.x = new_x;
        self.loc.y = new_y;
        info!("{} moved to {:?}", self.name, self.loc);
    }
    pub fn in_range(&self, target: &Location) -> bool {
        let range = self.loc.distance(target);
        range <= self.range.curr as f32
    }
    pub fn attack(&self, target: &Player, rng: &mut ThreadRng) -> bool {
        let roll = roll1d20(rng);
        self.attack.bonus() + roll >= target.defense.curr
    }
    pub fn damage(&self, target: &mut Player, rng: &mut ThreadRng) -> i32 {
        let damage_inflicted = roll1d8(rng) + self.power.bonus();
        target.armor.curr -= damage_inflicted;
        damage_inflicted
    }
    pub fn is_dead(&self) -> bool {
        self.armor.curr < 1
    }
}

pub struct Game {
    pub turns: i32,
    pub players: Vec<Player>
}

impl Game {
    pub fn new() -> Self {
        Game { turns: 0, players: Vec::new() }
    }
    pub fn get_nearest(&self, source: &Player) -> Option<&Player> {
        let mut min_distance = f32::MAX;
        let mut target: Option<&Player> = None;
        for player in &self.players {
            if source.name != player.name {
                let distance = source.loc.distance(&player.loc);
                if distance < min_distance {
                    min_distance = distance;
                    target = Option::Some(player);
                }
            }
        }
        target
    }
    pub fn get_nearest_idx(&self, source: &Player) -> Option<usize> {
        let mut min_distance = f32::MAX;
        let mut target: Option<usize> = None;
        for i in 0..self.players.len() {
            if source.name != self.players[i].name {
                let distance = source.loc.distance(&self.players[i].loc);
                if distance < min_distance {
                    min_distance = distance;
                    target = Option::Some(i);
                }
            }
        }
        target
    }
    pub fn run_simulation(&mut self, rng: &mut ThreadRng) -> i32 {
        while self.players.len() > 1 {
            for i in 0..self.players.len() {
                let target_idx = self.get_nearest_idx(&self.players[i]).unwrap();
                if self.players[i].in_range(&self.players[target_idx].loc) {
                    info!("{} is in range of {}", self.players[i].name, self.players[target_idx].name);
                    if self.players[i].attack(&self.players[target_idx], rng) {
                        self.players[i].damage(&mut self.players[target_idx], rng);
                        if self.players[target_idx].is_dead() {
                            warn!("{} defeated {}", self.players[i].name, self.players[target_idx].name);
                            self.players.remove(target_idx);
                        }
                    }
                }
                else {
                    info!("{} is moving towards {}", self.players[i].name, self.players[target_idx].name);
                }
            }
            self.turns += 1;
            if self.turns > MAXTURNS {
                warn!("Battle is taking too many turns: {}", self.turns);
                break;
            }
        }
        self.turns
    }
}

fn main() {
    // turn on loggging
    tracing_subscriber::fmt::init();
    info!("battlers/{}", VERSION);

    // initialize the random number generator
    let mut rng: ThreadRng = rand::rng();

    let mut p1 = Player::new("Fred");
    let mut p2 = Player::new("John");
    p1.randomize(&mut rng);
    p2.randomize(&mut rng);
    info!("{:?}", p1);
    info!("{:?}", p2);

    let mut game = Game::new();
    game.players.push(p1);
    game.players.push(p2);
    let turns_elapsed = game.run_simulation(&mut rng);
}
