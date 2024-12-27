use std::collections::HashSet;

use serde::Serialize;

/// Super Metroid's PRNG function.
///
/// A translation of https://patrickjohnston.org/bank/80?just=8111
pub fn rng1(seed: u16) -> u16 {
    let result = (seed & 0xFF) * 5;
    let hi = (((seed >> 8) & 0xFF) * 5) & 0xFF;
    let result = result as u32 + ((hi as u32) << 8) + 0x100;
    ((result >> 16) + result + 0x11) as u16
}

/// Represents the state & parameters of the random number generator.
#[derive(Clone, Debug, Serialize)]
pub struct Rng {
    /// The current seed value ($05E5).
    pub seed: u16,

    /// Whether the current room is an XBA room.
    ///
    /// In rooms with lava or acid, the low and high bytes of the random seed
    /// are swapped every frame.
    pub xba: bool,

    /// How many "extra" RNG calls per frame to simulate.
    ///
    /// Useful for simulating other enemies that may call RNG. Should usually be at least 1, since
    /// the game's main loop calls RNG once per frame.
    pub calls_per_frame: usize,
}

impl Rng {
    /// Returns the current value of the seed without updating it..
    pub fn read(&self) -> u16 {
        self.seed
    }

    /// Resets the RNG seed to a new value.
    pub fn reseed(&mut self, new_seed: u16) {
        self.seed = new_seed;
    }

    /// Returns another `Rng` instance with the same parameters, but a different seed.
    pub fn with_seed(&self, seed: u16) -> Rng {
        Rng { seed, ..*self }
    }

    /// Generates a new random number, updating the seed.
    pub fn roll(&mut self) -> u16 {
        self.seed = rng1(self.seed);
        self.seed
    }

    /// Advances to the next simulated frame by applying `calls_per_frame` and XBA.
    pub fn frame_advance(&mut self) {
        for _ in 0..self.calls_per_frame {
            self.roll();
        }
        if self.xba {
            self.seed = self.seed.swap_bytes();
        }
    }

    /// Returns an iterator over all seeds between the current state and the first repeated seed.
    pub fn seeds_until_loop(&self) -> impl Iterator<Item = u16> {
        struct State {
            seen: HashSet<u16>,
            cur: Rng,
        }
        impl Iterator for State {
            type Item = u16;

            fn next(&mut self) -> Option<Self::Item> {
                let seed = self.cur.seed;
                if self.seen.insert(seed) {
                    self.cur.frame_advance();
                    Some(seed)
                } else {
                    None
                }
            }
        }
        State {
            seen: HashSet::new(),
            cur: self.clone(),
        }
    }

    /// The RNG state after reset.
    pub const RESET: Rng = Rng {
        seed: 0x0061,
        xba: false,
        calls_per_frame: 1,
    };

    /// The RNG state after entering a room with a beetom.
    pub const BEETOM: Rng = Rng {
        seed: 0x0017,
        ..Rng::RESET
    };

    /// The RNG state after entering a room with a sidehopper.
    pub const SIDEHOPPER: Rng = Rng {
        seed: 0x0025,
        ..Rng::RESET
    };

    /// The RNG state after enteringa room with a polyp (the "lava rocks" in Volcano Room).
    ///
    /// Note that this state sets XBA to true, since the only room with polyps is an XBA room.
    pub const POLYP: Rng = Rng {
        seed: 0x0011,
        xba: true,
        ..Rng::RESET
    };
}
