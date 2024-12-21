#![allow(dead_code)]

mod loop_analysis;

trait RngFn: Fn(u16) -> u16 {}
impl<F: Fn(u16) -> u16> RngFn for F {}

fn rng1(seed: u16) -> u16 {
    let result = (seed & 0xFF) * 5;
    let hi = (((seed >> 8) & 0xFF) * 5) & 0xFF;
    let result = result as u32 + ((hi as u32) << 8) + 0x100;
    ((result >> 16) + result + 0x11) as u16
}

fn xba(seed: u16) -> u16 {
    seed.rotate_right(8)
}

fn inv(mut f: impl FnMut(u16) -> u16, y: u16) -> impl Iterator<Item = u16> {
    (0..=0xFFFF).filter(move |&x| f(x) == y)
}

fn main() {
    let rng = |x| xba(rng1(x));
    loop_analysis::analyze(rng).print();
}
