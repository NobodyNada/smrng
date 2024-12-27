use crate::Rng;

use super::{Drop, DropSet, DropTable};

/// The simulated results of farming an enemy across a set of seeds.
#[derive(Default, PartialEq, Eq, Hash)]
pub struct DropAnalysis {
    /// The number of seeds sampled.
    pub seeds: u32,

    pub nothing: u32,
    pub small_energy: u32,
    pub big_energy: u32,
    pub missile: u32,
    pub super_missile: u32,
    pub power_bomb: u32,
}

impl DropAnalysis {
    fn update(&mut self, drop: Drop) {
        match drop {
            Drop::Nothing => self.nothing += 1,
            Drop::SmallEnergy => self.small_energy += 1,
            Drop::BigEnergy => self.big_energy += 1,
            Drop::Missile => self.missile += 1,
            Drop::SuperMissile => self.super_missile += 1,
            Drop::PowerBomb => self.power_bomb += 1,
        }
    }
}

/// Generates a `DropAnalysis` for a set of seeds, simulating the actual RNG behavior (including
/// correlation between successive calls).
pub fn analyze_correlated(
    table: &DropTable,
    possible_drops: &DropSet,
    n: u32,
    rng: Rng,
    seeds: impl IntoIterator<Item = u16>,
) -> DropAnalysis {
    let mut analysis = DropAnalysis::default();

    for seed in seeds {
        let mut rng = rng.with_seed(seed);
        for drop in table.roll_multiple(&mut rng, possible_drops, n) {
            analysis.update(drop);
        }

        analysis.seeds += 1;
    }
    analysis
}

/// Generates a `DropAnalysis` for a set of seeds; simulating RNG distribution across the given set
/// of seeds, but assuming successive calls are independent.
pub fn analyze_uncorrelated<S: IntoIterator<Item = u16>>(
    table: &DropTable,
    possible_drops: &DropSet,
    n: u32,
    seeds: S,
) -> DropAnalysis
where
    S::IntoIter: ExactSizeIterator + Clone,
{
    let mut analysis = DropAnalysis::default();
    let seeds = seeds.into_iter();
    let num_seeds = seeds.len() as u32;
    analysis.seeds = num_seeds;

    let drop_count = table.count.unwrap_or(1) + table.extra.as_ref().map(|_| 1).unwrap_or(0);

    seeds
        .cycle()
        .take((num_seeds * drop_count * n) as usize)
        .map(|seed| Rng::RESET.with_seed(seed))
        .for_each(|mut rng| analysis.update(table.roll_one(&mut rng, possible_drops)));

    analysis
}
