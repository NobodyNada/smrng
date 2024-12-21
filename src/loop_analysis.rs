use super::RngFn;

pub struct Analysis<Rng: RngFn> {
    rng: Rng,
    seeds: [SeedInfo; 0x1000],
    branches: Vec<BranchInfo>,
    loops: Vec<LoopInfo>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SeedInfo {
    Branch { id: u16 },
    Loop { id: u16 },
}

pub struct BranchInfo {
    start: u16,
    length: u16,
    loop_id: u16,
}

pub struct LoopInfo {
    period: u16,
    start: u16,
}

pub fn analyze<Rng: RngFn>(rng: Rng) -> Analysis<Rng> {
    let mut seeds = [Option::<SeedInfo>::None; 0x10000];
    let mut branches = Vec::new();
    let mut loops = Vec::new();

    for start in 0..=0xFFFFu16 {
        if seeds[start as usize].is_some() {
            continue;
        }

        // Mark the generated values as a new branch
        let mut s = start;
        let mut length = 0;
        let new_branch = SeedInfo::Branch {
            id: branches.len() as u16,
        };
        while seeds[s as usize].is_none() {
            seeds[s as usize] = Some(new_branch);
            length += 1;
            s = rng(s);
        }

        match seeds[s as usize] {
            None => unreachable!(),
            Some(SeedInfo::Loop { id }) => {
                // We've found a new branch leading into an existing loop.
                branches.push(BranchInfo {
                    start,
                    length,
                    loop_id: id,
                });
            }
            Some(info) if info == new_branch => {
                // We've found a new loop, possibly with a new branch leading up to it.
                let new_loop = SeedInfo::Loop {
                    id: loops.len() as u16,
                };

                // Determine the length of the loop.
                let loop_start = s;
                let mut period = 1;
                s = rng(s);
                seeds[s as usize] = Some(new_loop);
                while s != loop_start {
                    period += 1;
                    s = rng(s);
                    seeds[s as usize] = Some(new_loop);
                }

                // Determine the length of the branch leading up to the loop.
                length = 0;
                s = start;
                while seeds[s as usize] == Some(new_branch) {
                    length += 1;
                    s = rng(s);
                }

                if length != 0 {
                    branches.push(BranchInfo {
                        start,
                        length,
                        loop_id: loops.len() as u16,
                    });
                }

                loops.push(LoopInfo {
                    period,
                    start: loop_start,
                })
            }
            suffix @ Some(SeedInfo::Branch { id }) => {
                // We've found a prefix of an existing branch.
                branches[id as usize].start = start;
                branches[id as usize].length += length;
                s = start;
                for _ in 0..length {
                    seeds[s as usize] = suffix;
                    s = rng(s);
                }
            }
        }
    }

    Analysis {
        rng,
        seeds: std::array::from_fn(|s| seeds[s].unwrap()),
        branches,
        loops,
    }
}

impl<Rng: RngFn> Analysis<Rng> {
    pub fn print(&self) {
        println!(
            "Loops: {} {:?}",
            self.loops.len(),
            self.loops.iter().map(|l| l.period).collect::<Vec<_>>()
        );
        println!(
            "Branches: {} {:#?}",
            self.branches.len(),
            self.branches
                .iter()
                .map(|b| format!("{} -> {}", b.length, b.loop_id))
                .collect::<Vec<_>>()
        );

        for (id, l) in self.loops.iter().enumerate() {
            println!();
            let mut s = l.start;
            if l.period > 100 {
                println!("Loop {id} (period {}) at {s:#06x}", l.period);
            } else {
                println!("Loop {id} (period {}):", l.period);
                print!("{s:#06x}");
                for _ in 1..l.period {
                    print!(", {s:#06x}");
                    s = (self.rng)(s);
                }
                println!();
            }
        }
    }
}
