use std::ops::Neg;
use rand::Rng;

pub trait RngExtras {
    fn plus_or_minus<N>(&mut self, value: N) -> N
        where N: Neg<Output = N>;
}
impl <R> RngExtras for R
    where R: Rng
{
    fn plus_or_minus<N: Neg>(&mut self, value: N) -> N
        where N: Neg<Output = N>
    {
        if self.gen_bool(0.5) {
            value
        } else {
            -value
        }
    }
}