// Based on the section "One-pass sampling" in Kirill Müller's paper
// "Accelerating weighted random sampling without

#[macro_use] extern crate failure;

use core::cmp::Ordering;
use std::collections::BinaryHeap;
use rand::Rng;
use rand::distributions::Distribution;

#[derive(Debug, Fail)]
#[fail(display = "Cannot sample over values with negative, NaN, or infinite weights.")]
pub struct HasInvalidWeights;

struct WsworEntry<T> {
    weight: f64,
    val: T,
}

impl<T> PartialOrd for WsworEntry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.weight.partial_cmp(&other.weight)
    }
}

impl<T> PartialEq for WsworEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.weight.eq(&other.weight)
    }
}

impl<T> Eq for WsworEntry<T> {
}

impl<T> Ord for WsworEntry<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.weight.partial_cmp(&other.weight).unwrap()
    }
}

pub fn wswor<T, I, R>(iter: I, rng: &mut R, count: usize) -> Result<Box<[T]>, HasInvalidWeights>
where I: Iterator<Item = (f64, T)>,
      R: Rng {
    use core::num::FpCategory::*;

    let mut heap = BinaryHeap::with_capacity(count + 1);
    let mut dist = rand::distributions::Exp1.sample_iter(rng);

    let mut cur_index = 0;

    for (w, v) in iter {
        match w.classify() {
            Nan | Infinite => Err(HasInvalidWeights)?,
            Zero => continue, // just skip ahead if it's a zero (side-effect)
            _ => {}
        }

        if w.is_sign_negative() {
            Err(HasInvalidWeights)?;
        }

        let entry = WsworEntry {
            weight: w * dist.next().unwrap(),
            val: v,
        };

        heap.push(entry);

        if cur_index >= count {
            heap.pop();
        }

        cur_index += 1;
    }

    // return only the values; take their weights away
    Ok(
        heap.into_iter()
            .map(|entry| entry.val)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    )
}
