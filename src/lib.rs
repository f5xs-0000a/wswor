// Based on the section "One-pass sampling" from Müller (2016).
//
// - Müller, K. (2016). Accelerating weighted random sampling without replacement. Arbeitsberichte Verkehrs- Und Raumplanung, 1141. https://www.research-collection.ethz.ch/mapping/view/pub:176429

use core::{
    cmp::Ordering,
    num::FpCategory::*,
};
use std::{
    collections::BinaryHeap,
    fmt::Display,
    error::Error,
};

use num::Float;
use rand::{
    distr::Distribution,
    RngCore,
};
use rand_distr::Exp1;

#[derive(Debug)]
pub enum HasInvalidWeights {
    NaN,
    Infinite,
    Negative,
}

impl Display for HasInvalidWeights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cannot sample over values with {} weights.", match self {
            HasInvalidWeights::NaN => "NaN",
            HasInvalidWeights::Infinite => "infinite",
            HasInvalidWeights::Negative => "negative",
        })
    }
}

impl Error for HasInvalidWeights {}

impl HasInvalidWeights {
    fn check_weight<F: Float>(weight: &F) -> Result<(), Self> {
        // infinite and NaNs are invalid weights
        match weight.classify() {
            Nan => Err(Self::NaN)?,
            Infinite => Err(Self::Infinite)?,
            _ => {},
        }

        // so are negative weights
        if weight.is_sign_negative() {
            Err(Self::Negative)?;
        }

        Ok(())
    }
}

struct WsworEntry<F: Float, T> {
    weight: F,
    val: T,
}

impl<F: Float, T> PartialOrd for WsworEntry<F, T> {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        self.weight.partial_cmp(&other.weight)
    }
}

impl<F: Float, T> PartialEq for WsworEntry<F, T>
where
    F: Float,
{
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.weight.eq(&other.weight)
    }
}

// we'll enforce total ordering ourselves; we can take care of this
impl<F: Float, T> Eq for WsworEntry<F, T> {}

impl<F: Float, T> Ord for WsworEntry<F, T> {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        self.weight.partial_cmp(&other.weight).unwrap()
    }
}

/// One-pass Weighted Random Sampler Without Replacement.
///
/// Can sample any number of elements.
pub struct StreamingWswor<F: Float, T> {
    count: usize,
    heap: BinaryHeap<WsworEntry<F, T>>,
}

impl<F, T> StreamingWswor<F, T>
where
    F: Float,
    Exp1: Distribution<F>,
{
    pub fn new(count: usize) -> StreamingWswor<F, T> {
        StreamingWswor {
            count,
            heap: BinaryHeap::with_capacity(count + 1),
        }
    }

    /// NOTE: the consumption of the iterator will be halted prematurely if an
    /// invalid weight is detected
    pub fn feed_iter<R: RngCore>(
        &mut self,
        iter: impl Iterator<Item = (F, T)>,
        rng: &mut R,
    ) -> Result<(), HasInvalidWeights> {
        for (w, v) in iter {
            self.feed(v, w, rng)?;
        }

        Ok(())
    }

    pub fn feed<R: RngCore>(
        &mut self,
        val: T,
        weight: F,
        rng: &mut R,
    ) -> Result<(), HasInvalidWeights> {
        HasInvalidWeights::check_weight(&weight)?;

        let mut dist = Exp1.sample_iter(rng);

        let entry = WsworEntry {
            val,
            weight: {
                if weight == F::zero() {
                    F::max_value()
                }
                else {
                    let random: F = dist.next().unwrap();
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

/// Special case for the One-pass Weighted Sampler where you just need one item
/// sampled.
pub struct SingleStreamingWs<F: Float, T> {
    value: Option<T>,
    exp_value_weight: F,
}

impl<F, T> SingleStreamingWs<F, T>
where
    F: Float,
    Exp1: Distribution<F>,
{
    pub fn new() -> SingleStreamingWs<F, T> {
        SingleStreamingWs {
            value: None,
            exp_value_weight: F::zero(),
        }
    }

    pub fn feed<R: RngCore>(
        &mut self,
        val: T,
        weight: F,
        rng: &mut R,
    ) -> Result<(), HasInvalidWeights> {
        HasInvalidWeights::check_weight(&weight)?;
        let mut dist = Exp1.sample_iter(rng);
        let exp_weight = dist.next().unwrap() / weight;

        if self.value.is_none() {
            self.value = Some(val);
            self.exp_value_weight = exp_weight
        }
        else if exp_weight < self.exp_value_weight {
            self.value = Some(val);
            self.exp_value_weight = exp_weight;
        }

        Ok(())
    }

    pub fn feed_iter<R: RngCore>(
        &mut self,
        iter: impl Iterator<Item = (F, T)>,
        rng: &mut R,
    ) -> Result<(), HasInvalidWeights> {
        for (w, v) in iter {
            self.feed(v, w, rng)?;
        }

        Ok(())
    }

    pub fn get(&self) -> Option<&T> {
        self.value.as_ref()
    }

    pub fn take(mut self) -> Option<T> {
        self.value.take()
    }
}

/// Quick and easy weighted random sampling without replacement.
pub fn wswor<F: Float, T, R: RngCore>(
    iter: impl Iterator<Item = (F, T)>,
    rng: &mut R,
    count: usize,
) -> Result<impl Iterator<Item = T>, HasInvalidWeights>
where
    F: Float,
    R: RngCore,
    Exp1: Distribution<F>,
{
    let mut heap = StreamingWswor::new(count);
    heap.feed_iter(iter, rng)?;
    Ok(heap.take())
}
