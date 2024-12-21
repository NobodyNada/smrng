use serde::Serialize;

pub fn rng1(seed: u16) -> u16 {
    let result = (seed & 0xFF) * 5;
    let hi = (((seed >> 8) & 0xFF) * 5) & 0xFF;
    let result = result as u32 + ((hi as u32) << 8) + 0x100;
    ((result >> 16) + result + 0x11) as u16
}

#[allow(dead_code)]
pub fn inv(mut f: impl FnMut(u16) -> u16, y: u16) -> impl Iterator<Item = u16> {
    (0..=0xFFFF).filter(move |&x| f(x) == y)
}

#[derive(Clone, Debug, Serialize)]
pub struct Rng {
    pub seed: u16,
    pub xba: bool,
    pub calls_per_frame: usize,
}

impl Rng {
    pub fn read(&self) -> u16 {
        self.seed
    }

    pub fn reseed(&mut self, new_seed: u16) {
        self.seed = new_seed;
    }

    pub fn with_seed(&self, seed: u16) -> Rng {
        Rng { seed, ..*self }
    }

    pub fn roll(&mut self) -> u16 {
        self.seed = rng1(self.seed);
        self.seed
    }

    pub fn frame_advance(&mut self) {
        for _ in 0..self.calls_per_frame {
            self.roll();
        }
        if self.xba {
            self.seed = self.seed.swap_bytes();
        }
    }

    pub const RESET: Rng = Rng {
        seed: 0x0061,
        xba: false,
        calls_per_frame: 1,
    };

    pub const BEETOM: Rng = Rng {
        seed: 0x0017,
        ..Rng::RESET
    };
    pub const SIDEHOPPER: Rng = Rng {
        seed: 0x0025,
        ..Rng::RESET
    };
    pub const POLYP: Rng = Rng {
        seed: 0x0011,
        xba: true,
        ..Rng::RESET
    };
}
