use serde::Serialize;

use crate::Rng;

/// The structure of RNG loops and branches given a particular RNG configuration.
///
/// Every RNG seed can be classified as part of either a loop (a cyclic set of seeds) or a branch
/// (eventually leading into a loop).
#[derive(Serialize)]
pub struct Analysis {
    /// The RNG configuration under analysis.
    pub rng: Rng,

    /// The behavior of all possible seeds with this RNG configuration.
    pub seeds: Vec<SeedInfo>,

    /// A list of all RNG branches.
    pub branches: Vec<BranchInfo>,

    /// A list of all RNG seeds.
    pub loops: Vec<LoopInfo>,
}

/// Whether a given RNG seed is a branch or a loop.
#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SeedInfo {
    Branch { id: u16 },
    Loop { id: u16 },
}

/// A branch is a set of RNG seeds that are not themselves part of a loop,
/// but eventually lead into one.
#[derive(Serialize)]
pub struct BranchInfo {
    pub seeds: Vec<u16>,
    pub loop_id: u16,
}

/// A loop is a set of RNG seeds that form a cycle.
#[derive(Serialize)]
pub struct LoopInfo {
    pub seeds: Vec<u16>,
}

impl Rng {
    /// Performs loop analysis on this RNG to determine all possible loops and branches.
    pub fn analyze(&self) -> Analysis {
        let mut seeds = [Option::<SeedInfo>::None; 0x10000];
        let mut branches = Vec::new();
        let mut loops = Vec::new();

        // Check the starting seed first, so that it gets assigned branch 0 and loop 0.
        for start in std::iter::once(self.seed).chain(0..=0xFFFFu16) {
            if seeds[start as usize].is_some() {
                continue;
            }

            // Mark the generated values as a new branch
            let mut rng = self.with_seed(start);
            let mut seeds_seen = Vec::new();
            let new_branch = SeedInfo::Branch {
                id: branches.len() as u16,
            };
            while seeds[rng.seed as usize].is_none() {
                seeds[rng.seed as usize] = Some(new_branch);
                seeds_seen.push(rng.seed);
                rng.frame_advance();
            }

            match seeds[rng.seed as usize] {
                None => unreachable!(),
                Some(SeedInfo::Loop { id }) => {
                    // We've found a new branch leading into an existing loop.
                    branches.push(BranchInfo {
                        seeds: seeds_seen,
                        loop_id: id,
                    });
                }
                Some(info) if info == new_branch => {
                    // We've found a new loop, possibly with a new branch leading up to it.
                    let new_loop = SeedInfo::Loop {
                        id: loops.len() as u16,
                    };

                    // Determine the length of the loop.
                    let (branch_seeds, loop_seeds) = seeds_seen.split_at(
                        seeds_seen
                            .iter()
                            .enumerate()
                            .find(|(_, seed)| **seed == rng.seed)
                            .unwrap()
                            .0,
                    );
                    for &seed in branch_seeds {
                        seeds[seed as usize] = Some(new_branch);
                    }
                    for &seed in loop_seeds {
                        seeds[seed as usize] = Some(new_loop);
                    }

                    if !branch_seeds.is_empty() {
                        branches.push(BranchInfo {
                            seeds: branch_seeds.to_vec(),
                            loop_id: loops.len() as u16,
                        });
                    }

                    loops.push(LoopInfo {
                        seeds: loop_seeds.to_vec(),
                    })
                }
                suffix @ Some(SeedInfo::Branch { id }) => {
                    // We've found a prefix of an existing branch.
                    for &seed in &seeds_seen {
                        seeds[seed as usize] = suffix;
                    }
                    let branch = &mut branches[id as usize];
                    seeds_seen.append(&mut branch.seeds);
                    branches[id as usize].seeds = seeds_seen;
                    rng.reseed(start);
                }
            }
        }

        Analysis {
            rng: self.clone(),
            seeds: seeds.into_iter().map(Option::unwrap).collect(),
            branches,
            loops,
        }
    }
}

impl Analysis {
    pub fn print(&self) {
        println!("Loop analysis for {:#?}", self.rng);
        println!();
        for (id, l) in self.loops.iter().enumerate() {
            let start = l.seeds[0];
            let period = l.seeds.len();
            if period > 100 {
                println!("Loop {id} (period {period}) at {start:#06x}");
            } else {
                println!("Loop {id} (period {period}):");

                const PER_LINE: usize = 10;
                for (i, seed) in l.seeds.iter().enumerate() {
                    if i % PER_LINE == 0 {
                        print!("    ");
                    }
                    print!("{:#06x}", seed);
                    if i == period - 1 || (i + 1) % PER_LINE == 0 {
                        println!();
                    } else {
                        print!(", ");
                    }
                }
            }
        }
        println!();
        println!("Branches: {}", self.branches.len());
        for (i, branch) in self.branches.iter().enumerate() {
            let pad = self.branches.len().ilog10() as usize + 1;
            println!(
                "    {i:pad$}: length {:5} -> loop {}",
                branch.seeds.len(),
                branch.loop_id
            );
        }
    }
}
