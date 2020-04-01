// Based on the section "One-pass sampling" in Kirill Müller's paper
// "Accelerating weighted random sampling without

#[macro_use]
extern crate failure;

////////////////////////////////////////////////////////////////////////////////

use core::{
    cmp::Ordering,
    num::FpCategory::*,
};
use rand::{
    distributions::Distribution,
    RngCore,
};
use std::collections::BinaryHeap;
use rand_distr::Exp1;

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Fail)]
#[fail(display = "Cannot sample over values with negative, NaN, or infinite \
                  weights.")]
pub struct HasInvalidWeights;

struct WsworEntry<T> {
    weight: f64,
    val:    T,
}

impl<T> PartialOrd for WsworEntry<T> {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering>
    {
        self.weight.partial_cmp(&other.weight)
    }
}

impl<T> PartialEq for WsworEntry<T> {
    fn eq(
        &self,
        other: &Self,
    ) -> bool
    {
        self.weight.eq(&other.weight)
    }
}

impl<T> Eq for WsworEntry<T> {
}

impl<T> Ord for WsworEntry<T> {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering
    {
        self.weight.partial_cmp(&other.weight).unwrap()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct StreamingWswor<T> {
    count: usize,
    heap:  BinaryHeap<WsworEntry<T>>,
}

impl<T> StreamingWswor<T> {
    pub fn new(count: usize) -> StreamingWswor<T> {
        StreamingWswor {
            count,
            heap: BinaryHeap::with_capacity(count + 1),
        }
    }

    /// NOTE: the consumption of the iterator will be halted prematurely if an
    /// invalid weight is detected
    pub fn feed_iter<R: RngCore>(
        &mut self,
        iter: impl Iterator<Item = (f64, T)>,
        rng: &mut R,
    ) -> Result<(), HasInvalidWeights>
    {
        for (w, v) in iter {
            self.feed(v, w, rng)?;
        }

        Ok(())
    }

    pub fn feed<R: RngCore>(
        &mut self,
        val: T,
        weight: f64,
        rng: &mut R,
    ) -> Result<(), HasInvalidWeights>
    {
        match weight.classify() {
            Nan | Infinite => Err(HasInvalidWeights)?,
            _ => {},
        }

        if weight.is_sign_negative() {
            Err(HasInvalidWeights)?;
        }

        let mut dist = Exp1.sample_iter(rng);

        let entry = WsworEntry {
            val,
            weight: {
                if weight == 0. {
                    core::f64::MIN
                }
                else {
                    let random: f64 = dist.next().unwrap();
                    random / weight
                }
            },
        };

        self.heap.push(entry);

        if self.heap.len() > self.count {
            self.heap.pop();
        }

        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.heap.iter().map(|entry| &entry.val)
    }

    pub fn take(self) -> impl Iterator<Item = T> {
        self.heap.into_iter().map(|entry| entry.val)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct SingleStreamingWs<T>(StreamingWswor<T>);

impl<T> SingleStreamingWs<T> {
    pub fn new() -> SingleStreamingWs<T> {
        SingleStreamingWs(StreamingWswor::new(1))
    }

    pub fn feed<R: RngCore>(
        &mut self,
        val: T,
        weight: f64,
        rng: &mut R,
    ) -> Result<(), HasInvalidWeights>
    {
        self.0.feed(val, weight, rng)
    }

    pub fn feed_iter<R: RngCore>(
        &mut self,
        iter: impl Iterator<Item = (f64, T)>,
        rng: &mut R,
    ) -> Result<(), HasInvalidWeights>
    {
        self.0.feed_iter(iter, rng)
    }

    pub fn get(&self) -> Option<&T> {
        self.0.iter().next()
    }

    pub fn take(self) -> Option<T> {
        self.0.take().next()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn wswor<T, R>(
    iter: impl Iterator<Item = (f64, T)>,
    rng: &mut R,
    count: usize,
) -> Result<impl Iterator<Item = T>, HasInvalidWeights>
where
    R: RngCore,
{
    let mut heap = StreamingWswor::new(count);
    heap.feed_iter(iter, rng)?;
    Ok(heap.take())
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "gitversion")]
pub fn git_version() -> &'static str {
    git_version::git_version!()
}
