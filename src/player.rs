use core::f32;
use rand::{Rng, rngs::ThreadRng};
use tracing::{debug, warn};

use crate::dice;

#[derive(Debug)]
pub enum Attribute {
    Attack,
    Defense,
    Armor,
    Power,
    Speed,
    Range,
}

#[derive(Debug)]
pub struct PlayerAttribute {
    pub name: Attribute,
    pub base: i32,
    pub curr: i32,
}

impl PlayerAttribute {
    pub fn new(name: Attribute) -> Self {
        PlayerAttribute {
            name: name,
            base: 0,
            curr: 0,
        }
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
        self.base = dice::roll3d6(rng);
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
        let roll_x = rng.random_range(1..=60);
        let roll_y = rng.random_range(1..=60);
        self.x = roll_x as f32;
        self.y = roll_y as f32;
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
    pub loc: Location,
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
            loc: Location::new(0.0, 0.0, 0.0),
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
        debug!("{}:{} moved to {}:{}", self.loc.x, self.loc.y, new_x, new_y);
        self.loc.x = new_x;
        self.loc.y = new_y;
    }
    pub fn in_range(&self, target: &Location) -> bool {
        let range = self.loc.distance(target);
        range <= self.range.curr as f32
    }
    pub fn attack(&self, target: &Player, rng: &mut ThreadRng) -> bool {
        let roll = dice::roll1d20(rng);
        self.attack.bonus() + roll >= target.defense.curr
    }
    pub fn damage(&self, target: &mut Player, rng: &mut ThreadRng) -> i32 {
        let damage_inflicted = dice::roll1d8(rng) + self.power.bonus();
        if damage_inflicted < 1 {
            warn!("no damage inflicted!");
            return 0
        }
        target.armor.curr -= damage_inflicted;
        damage_inflicted
    }
    pub fn is_dead(&self) -> bool {
        self.armor.curr < 1
    }
}
