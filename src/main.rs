use std::{num::ParseIntError, process::exit};

use clap::{builder::BoolishValueParser, Parser, Subcommand};
pub use rng::Rng;

mod drops;
mod loop_analysis;
mod rng;

#[derive(Parser, Debug)]
struct Args {
    /// Whether to simulate RNG behavior in an XBA room.
    #[arg(short, long, num_args = 0..=1, default_missing_value = "true", value_parser = BoolishValueParser::new(), global = true)]
    xba: Option<bool>,

    /// How many RNG calls to simulate per frame.
    #[arg(short = 'n', long, default_value = "1", global = true)]
    calls_per_frame: usize,

    /// The initial seed value. Can be a number, or
    #[arg(short, long, value_parser = parse_seed, default_value = "reset", global = true)]
    seed: Rng,

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
        let mut rng = self.seed.clone();
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
    }
}
