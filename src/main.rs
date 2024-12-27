use ::smrng::drops::{analysis::DropAnalysis, Drop, DropSet};
use ::smrng::*;

use clap::{builder::BoolishValueParser, Parser, Subcommand};
use std::{collections::HashMap, num::ParseIntError, process::exit};

#[derive(Parser, Debug)]
struct Args {
    /// Whether to simulate RNG behavior in an XBA room.
    #[arg(short, long, num_args = 0..=1, default_missing_value = "true", value_parser = BoolishValueParser::new(), global = true)]
    xba: Option<bool>,

    /// How many RNG calls to simulate per frame.
    #[arg(short = 'n', long, default_value = "1", global = true)]
    calls_per_frame: usize,

    /// The initial seed value. Can be a number, or 'reset', 'beetom', 'sidehopper', or 'polyp'.
    /// Defaults to 'reset'.
    #[arg(short, long, value_parser = parse_seed, global = true)]
    seed: Option<Rng>,

    #[command(subcommand)]
    command: Command,
}

fn parse_seed(seed: &str) -> Result<Rng, ParseIntError> {
    match seed.to_lowercase().as_str() {
        "reset" => Ok(Rng::RESET),
        s if s.starts_with("power") => Ok(Rng::RESET),

        "beetom" => Ok(Rng::BEETOM),
        "sidehopper" | "hopper" => Ok(Rng::SIDEHOPPER),
        "polyp" => Ok(Rng::POLYP),

        n if n.starts_with("0x") => Ok(Rng {
            seed: u16::from_str_radix(&n[2..], 16)?,
            ..Rng::RESET
        }),
        n => Ok(Rng {
            seed: n.parse()?,
            ..Rng::RESET
        }),
    }
}

impl Args {
    fn rng(&self) -> Rng {
        let mut rng = self.seed.clone().unwrap_or(Rng::RESET);
        rng.calls_per_frame = self.calls_per_frame;
        rng.xba = self.xba.unwrap_or(rng.xba);
        rng
    }
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Print information about RNG loops and branches.
    Loops {
        /// Output in JSON format.
        #[arg(short, long)]
        json: bool,
    },

    /// Print generated random numbers to standard output.
    Dump {
        #[arg(
            short,
            long = "loop",
            name = "loop",
            default_missing_value = "0",
            num_args = 0..=1
        )]
        /// Dump only the values that are part of an RNG loop.
        /// You can optionally specify a loop ID as returned by `rng loops`.
        loop_id: Option<usize>,

        /// Dump only the values that are part of branch <BRANCH>, as returned by `rng loops`.
        #[arg(short, long, conflicts_with = "loop")]
        branch: Option<usize>,

        /// Output numbers in hexadecimal.
        #[arg(long)]
        hex: bool,
    },

    /// Print drop chances for an enemy
    Drops {
        /// How many of the enemy are killed with a single shot.
        #[arg(short, long, default_value = "1")]
        count: u32,

        /// Whether to ignore correlation between consecutive RNG calls.
        #[arg(long)]
        uncorrelated: bool,

        /// Whether to pretend the game generates drops uniformly, instead of using a PRNG.
        #[arg(long, conflicts_with = "uncorrelated")]
        ideal: bool,

        /// Output a histogram of drop chances instead of probabilities.
        #[arg(long, conflicts_with = "uncorrelated", conflicts_with = "ideal")]
        histogram: bool,

        /// Only consider RNG seeds that are part of a loop.
        /// You can optionally specify a loop ID as returned by `rng loops`.
        ///
        /// If no seed is specified, loop 0 of the reset seed is assumed automatically (the 2280 loop).
        #[arg(
            short,
            long = "loop",
            name = "loop",
            default_missing_value = "0",
            num_args = 0..=1
        )]
        loop_id: Option<usize>,

        /// Only consider RNG seeds that are part of branch <BRANCH>, as returned by `rng loops`.
        #[arg(short, long, conflicts_with = "loop")]
        branch: Option<usize>,

        /// Consider all 65536 RNG seeds, rather than just descendents of a given seed.
        ///
        /// In most circumstances, you want `--all_seeds` when you use `--xba`.
        #[arg(short, long, conflicts_with = "branch", conflicts_with = "loop")]
        all_seeds: bool,

        /// The player is full on energy.
        #[arg(short = 'e')]
        full_energy: bool,

        /// The player is full on missiles.
        #[arg(short = 'm', long)]
        full_missiles: bool,

        /// The player is full on super missiles.
        #[arg(short = 'u', long)]
        full_supers: bool,

        /// The player is full on power bombs.
        #[arg(short = 'p', long)]
        full_pbs: bool,

        /// The enemy name.
        enemy: String,
    },
}

fn main() {
    let args = Args::parse();
    match args.command {
        Command::Loops { json } => {
            let analysis = args.rng().analyze();
            if json {
                println!("{}", serde_json::to_string_pretty(&analysis).unwrap());
            } else {
                analysis.print();
            }
        }
        Command::Dump {
            loop_id,
            branch,
            hex,
        } => {
            let print = |seed| {
                if hex {
                    println!("{seed:#06x}");
                } else {
                    println!("{seed}");
                }
            };

            if let Some(loop_id) = loop_id {
                let analysis = args.rng().analyze();
                let Some(l) = analysis.loops.get(loop_id) else {
                    eprintln!("Loop index out of range 0..={}", analysis.loops.len());
                    exit(2);
                };
                l.seeds.iter().copied().for_each(print);
            } else if let Some(branch_id) = branch {
                let analysis = args.rng().analyze();
                let Some(b) = analysis.branches.get(branch_id) else {
                    eprintln!("Branch index out of range 0..={}", analysis.branches.len());
                    exit(2);
                };
                b.seeds.iter().copied().for_each(print);
            } else {
                let mut seen = vec![false; 0x10000];
                let mut rng = args.rng();

                while !seen[rng.seed as usize] {
                    print(rng.seed);
                    seen[rng.seed as usize] = true;
                    rng.frame_advance();
                }
            }
        }
        Command::Drops {
            count,
            uncorrelated,
            ideal,
            histogram,
            mut loop_id,
            branch,
            all_seeds,
            ref enemy,
            full_energy,
            full_missiles,
            full_supers,
            full_pbs,
        } => {
            let Some(drop_table) = drops::ENEMY_DROPS.get(enemy) else {
                eprintln!("Unknown enemy {enemy}");
                exit(2)
            };

            if loop_id.is_none() && branch.is_none() && !all_seeds && args.seed.is_none() {
                loop_id = Some(0);
            }
            let rng = args.rng();

            let seeds: Vec<u16> = if all_seeds {
                (0..=u16::MAX).collect()
            } else if let Some(loop_id) = loop_id {
                let mut analysis = args.rng().analyze();
                let Some(l) = analysis.loops.get_mut(loop_id) else {
                    eprintln!("Loop index out of range 0..={}", analysis.loops.len());
                    exit(2);
                };
                std::mem::take(&mut l.seeds)
            } else if let Some(branch_id) = branch {
                let mut analysis = args.rng().analyze();
                let Some(b) = analysis.branches.get_mut(branch_id) else {
                    eprintln!("Branch index out of range 0..={}", analysis.branches.len());
                    exit(2);
                };
                std::mem::take(&mut b.seeds)
            } else {
                rng.seeds_until_loop().collect()
            };

            let mut possible_drops = DropSet::ALL;
            if full_energy {
                possible_drops -= &DropSet::from_iter([Drop::SmallEnergy, Drop::BigEnergy]);
            }
            if full_missiles {
                possible_drops -= &DropSet::from_iter([Drop::Missile]);
            }
            if full_supers {
                possible_drops -= &DropSet::from_iter([Drop::SuperMissile]);
            }
            if full_pbs {
                possible_drops -= &DropSet::from_iter([Drop::PowerBomb]);
            }

            if histogram {
                let mut histogram = HashMap::<DropAnalysis, u32>::new();
                for &seed in &seeds {
                    let analysis = drops::analysis::analyze_correlated(
                        drop_table,
                        &possible_drops,
                        count,
                        rng.clone(),
                        std::iter::once(seed),
                    );
                    *histogram.entry(analysis).or_default() += 1;
                }

                let mut histogram: Vec<_> = histogram.into_iter().collect();
                histogram.sort_by_key(|(_, count)| u32::MAX - *count);

                println!("#            | Small E|   Big E| Missile|   Super|      PB");
                println!("-------------+--------+--------+--------+--------+--------");
                for (entry, count) in histogram {
                    println!(
                        "{:>5} ({}%)|{:>8}|{:>8}|{:>8}|{:>8}|{:>8}",
                        count,
                        format_percentage(count, seeds.len() as u32),
                        entry.small_energy,
                        entry.big_energy,
                        entry.missile,
                        entry.super_missile,
                        entry.power_bomb,
                    )
                }
            } else if ideal {
                let print_stat = |name, drop| {
                    let prob = drop_table.ideal_drops_per_farm(drop, &possible_drops, count);
                    println!("{name:>8} | {prob:.3}");
                };

                println!("Resource | Drops");
                println!("---------+------");
                print_stat("Small E", Drop::SmallEnergy);
                print_stat("Big E", Drop::BigEnergy);
                print_stat("Missile", Drop::Missile);
                print_stat("Super", Drop::SuperMissile);
                print_stat("PB", Drop::PowerBomb);
            } else {
                let analysis = if uncorrelated {
                    drops::analysis::analyze_uncorrelated(drop_table, &possible_drops, count, seeds)
                } else {
                    drops::analysis::analyze_correlated(
                        drop_table,
                        &possible_drops,
                        count,
                        rng.clone(),
                        seeds,
                    )
                };

                let print_stat =
                    |name, stat| println!("{name:>8} | {:.3}", stat as f32 / analysis.seeds as f32);

                println!("Resource | Drops");
                println!("---------+------");
                print_stat("Small E", analysis.small_energy);
                print_stat("Big E", analysis.big_energy);
                print_stat("Missile", analysis.missile);
                print_stat("Super", analysis.super_missile);
                print_stat("PB", analysis.power_bomb);
            }
        }
    }
}

fn format_percentage(num: u32, denom: u32) -> String {
    let percentage = (num as f32) / (denom as f32) * 100.;

    let digits_before_decimal = (percentage.floor() + 0.1).log10().max(1.).ceil() as usize;
    let digits_after_decimal = 3 - digits_before_decimal;
    format!(
        "{:b$.a$}",
        percentage,
        b = digits_before_decimal,
        a = digits_after_decimal
    )
}
