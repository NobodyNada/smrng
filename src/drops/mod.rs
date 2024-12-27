pub mod analysis;

use std::{
    collections::HashMap,
    ops::{BitAnd, BitOr, BitXor, Index, Sub},
    sync::LazyLock,
};

use serde::Deserialize;

use crate::Rng;

const ENEMY_DROPS_JSON: &str = include_str!("enemy_drops.json");

/// The drop table for enemies in vanilla SM.
pub static ENEMY_DROPS: LazyLock<HashMap<String, DropTable>> =
    LazyLock::new(|| serde_json::from_str(ENEMY_DROPS_JSON).unwrap());

/// A drop type.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Drop {
    Nothing,
    SmallEnergy,
    BigEnergy,
    Missile,
    SuperMissile,
    PowerBomb,
}

impl Drop {
    /// Whether this is considered a "Tier 2" drop.
    pub const fn is_major(&self) -> bool {
        use self::Drop::*;
        match self {
            Nothing | SmallEnergy | BigEnergy | Missile => false,
            SuperMissile | PowerBomb => true,
        }
    }

    const fn index(&self) -> u8 {
        use self::Drop::*;
        match self {
            SmallEnergy => 0,
            BigEnergy => 1,
            Missile => 2,
            Nothing => 3,
            SuperMissile => 4,
            PowerBomb => 5,
        }
    }

    const fn from_index(index: u8) -> Self {
        use self::Drop::*;
        match index {
            0 => SmallEnergy,
            1 => BigEnergy,
            2 => Missile,
            3 => Nothing,
            4 => SuperMissile,
            5 => PowerBomb,
            _ => panic!("invalid drop index"),
        }
    }
}

/// A set of drops.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DropSet(u8);

impl DropSet {
    pub const EMPTY: DropSet = DropSet::new();
    pub const ALL: DropSet = DropSet::from_slice(&[
        Drop::Nothing,
        Drop::SmallEnergy,
        Drop::BigEnergy,
        Drop::Missile,
        Drop::SuperMissile,
        Drop::PowerBomb,
    ]);
    pub const MINOR: DropSet = DropSet::from_slice(&[
        Drop::Nothing,
        Drop::SmallEnergy,
        Drop::BigEnergy,
        Drop::Missile,
    ]);
    pub const MAJOR: DropSet = DropSet::from_slice(&[Drop::SuperMissile, Drop::PowerBomb]);
    pub const HEALTH_BOMB: DropSet = DropSet::from_slice(&[Drop::SmallEnergy, Drop::BigEnergy]);

    pub const fn new() -> DropSet {
        DropSet(0)
    }

    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub const fn len(&self) -> usize {
        self.0.count_ones() as usize
    }

    pub const fn contains(&self, drop: &Drop) -> bool {
        self.0 & (1 << drop.index()) != 0
    }

    pub const fn insert(&mut self, drop: Drop) -> bool {
        let inserted = !self.contains(&drop);
        self.0 |= 1 << drop.index();
        inserted
    }

    pub const fn remove(&mut self, drop: Drop) -> bool {
        let removed = self.contains(&drop);
        self.0 &= !(1 << drop.index());
        removed
    }

    pub const fn intersection(&self, other: &DropSet) -> DropSet {
        DropSet(self.0 & other.0)
    }

    pub const fn union(&self, other: &DropSet) -> DropSet {
        DropSet(self.0 | other.0)
    }

    pub const fn difference(&self, other: &DropSet) -> DropSet {
        DropSet(self.0 & !other.0)
    }

    pub const fn symmetric_difference(&self, other: &DropSet) -> DropSet {
        DropSet(self.0 ^ other.0)
    }

    pub fn iter(&self) -> impl Iterator<Item = Drop> + '_ {
        DropSetIterator(self.clone())
    }

    const fn from_slice(drops: &[Drop]) -> DropSet {
        let mut result = DropSet::new();
        let mut i = 0;
        while i < drops.len() {
            result.insert(drops[i]);
            i += 1;
        }
        result
    }
}
impl Default for DropSet {
    fn default() -> Self {
        DropSet::new()
    }
}

impl BitAnd<&DropSet> for DropSet {
    type Output = DropSet;
    fn bitand(self, rhs: &DropSet) -> Self::Output {
        self.intersection(rhs)
    }
}
impl BitOr<&DropSet> for DropSet {
    type Output = DropSet;
    fn bitor(self, rhs: &DropSet) -> Self::Output {
        self.union(rhs)
    }
}
impl BitXor<&DropSet> for DropSet {
    type Output = DropSet;
    fn bitxor(self, rhs: &DropSet) -> Self::Output {
        self.symmetric_difference(rhs)
    }
}
impl Sub<&DropSet> for DropSet {
    type Output = DropSet;
    fn sub(self, rhs: &DropSet) -> Self::Output {
        self.difference(rhs)
    }
}

impl IntoIterator for DropSet {
    type Item = Drop;
    type IntoIter = DropSetIterator;
    fn into_iter(self) -> Self::IntoIter {
        DropSetIterator(self)
    }
}

impl Extend<Drop> for DropSet {
    fn extend<T: IntoIterator<Item = Drop>>(&mut self, iter: T) {
        for drop in iter {
            self.insert(drop);
        }
    }
}

impl FromIterator<Drop> for DropSet {
    fn from_iter<T: IntoIterator<Item = Drop>>(iter: T) -> Self {
        let mut result = DropSet::new();
        result.extend(iter);
        result
    }
}

pub struct DropSetIterator(DropSet);
impl Iterator for DropSetIterator {
    type Item = Drop;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            None
        } else {
            let drop = Drop::from_index(self.0 .0.trailing_zeros() as u8);
            self.0.remove(drop);
            Some(drop)
        }
    }
}

/// A table of an enemy's drop chances.
#[derive(Deserialize)]
pub struct DropTable {
    pub nothing: u8,
    pub small_energy: u8,
    pub big_energy: u8,
    pub missile: u8,
    pub super_missile: u8,
    pub power_bomb: u8,

    /// If this enemy calls a multi-drop routine, the number of drops to generate.
    #[serde(default)]
    pub count: Option<u32>,

    /// The type of explosion animation, if this enemy's explosion generates an extra drop.
    #[serde(default)]
    pub extra: Option<ExplosionDrop>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExplosionDrop {
    Metroid,
    Minikraid,
}

impl ExplosionDrop {
    /// The number of frames between explosions.
    pub fn explosion_interval(&self) -> u32 {
        8
    }

    /// The number of explosions before generating the final drop.
    pub fn num_explosions(&self) -> u32 {
        match self {
            ExplosionDrop::Metroid => 5,
            ExplosionDrop::Minikraid => 16,
        }
    }

    /// The number of RNG calls per explosion.
    pub fn rng_per_explosion(&self) -> u32 {
        match self {
            ExplosionDrop::Metroid => 2,
            ExplosionDrop::Minikraid => 3,
        }
    }
}

impl Index<self::Drop> for DropTable {
    type Output = u8;

    fn index(&self, index: self::Drop) -> &Self::Output {
        match index {
            Drop::Nothing => &self.nothing,
            Drop::SmallEnergy => &self.small_energy,
            Drop::BigEnergy => &self.big_energy,
            Drop::Missile => &self.missile,
            Drop::SuperMissile => &self.super_missile,
            Drop::PowerBomb => &self.power_bomb,
        }
    }
}

impl DropTable {
    /// Returns the raw drop chance for a given drop.
    pub fn get(&self, drop: Drop) -> u8 {
        self[drop]
    }

    /// Calculates ideal drops based purely on probabilities in the drop table.
    ///
    /// Returns the expected number of times `drop` will be dropped after farming this enemy
    /// `farms` times.
    pub fn ideal_drops_per_farm(&self, drop: Drop, possible_drops: &DropSet, farms: u32) -> f32 {
        if !possible_drops.contains(&drop) {
            return 0.;
        }

        let pooled_minor = possible_drops
            .intersection(&DropSet::MINOR)
            .iter()
            .map(|d| self[d])
            .sum::<u8>() as u16;

        let pooled_major_complement = 0xFF
            - possible_drops
                .intersection(&DropSet::MAJOR)
                .iter()
                .map(|d| self[d])
                .sum::<u8>() as u16;

        let chance = if drop.is_major() {
            self[drop] as u16 * pooled_major_complement / pooled_minor
        } else {
            self[drop] as u16
        };

        chance as f32 / 255.
            * (self.count.unwrap_or(1) + self.extra.as_ref().map(|_| 1).unwrap_or(0)) as f32
            * farms as f32
    }

    /// Simulates a single drop (even if this enemy drops multiple items).
    pub fn roll_one(&self, rng: &mut Rng, possible_drops: &DropSet) -> Drop {
        let random = loop {
            match rng.roll() as u8 {
                0 => continue,
                n => break n as u16,
            }
        };

        let pooled_minor = possible_drops
            .intersection(&DropSet::MINOR)
            .iter()
            .map(|d| self[d])
            .sum::<u8>() as u16;

        let pooled_major_complement = 0xFF
            - possible_drops
                .intersection(&DropSet::MAJOR)
                .iter()
                .map(|d| self[d])
                .sum::<u8>() as u16;

        let mut acc = 0;
        for drop in DropSet::MINOR {
            acc += (self[drop] as u16) * pooled_major_complement / pooled_minor;
            if acc >= random {
                return drop;
            }
        }

        for drop in DropSet::MAJOR {
            acc += self[drop] as u16;
            if acc >= random {
                return drop;
            }
        }

        Drop::Nothing
    }

    /// Simulates this enemy's drops.
    pub fn roll<'a>(
        &'a self,
        rng: &'a mut Rng,
        possible_drops: &'a DropSet,
    ) -> impl Iterator<Item = Drop> + 'a {
        self.roll_multiple(rng, possible_drops, 1)
    }

    /// Simulates the drops obtained by farming multiple of this enemy in a single frame.
    pub fn roll_multiple<'a>(
        &'a self,
        rng: &'a mut Rng,
        possible_drops: &'a DropSet,
        n: u32,
    ) -> impl Iterator<Item = Drop> + 'a {
        let mut main_drops = 0;
        let mut extra_drops = 0;
        std::iter::from_fn(move || {
            let count = self.count.unwrap_or(1) * n;

            if main_drops < count {
                main_drops += 1;
                rng.roll();
                Some(self.roll_one(rng, possible_drops))
            } else if let Some(extra) = &self.extra {
                if extra_drops < n {
                    if extra_drops == 0 {
                        for _ in 0..extra.num_explosions() {
                            for _ in 0..extra.rng_per_explosion() * n {
                                rng.roll();
                            }

                            for _ in 0..extra.explosion_interval() {
                                rng.frame_advance();
                            }
                        }
                    }

                    extra_drops += 1;
                    Some(self.roll_one(rng, possible_drops))
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}
