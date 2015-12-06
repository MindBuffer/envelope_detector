//! Peak envelope detection over a signal.
//!
//! The primary type of interest in this module is the [**Peak**](./struct.Peak) type, generic
//! over any [**Rectifier**](./trait.Rectifier).

use std::marker::PhantomData;


/// A rectifier that produces only the positive samples from a signal.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PositiveHalfWave {}
/// A rectifier that produces only the negative samples from a signal.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NegativeHalfWave {}
/// A rectifier that produces the absolute amplitude from samples from a signal.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FullWave {}


/// Types that can rectify some incoming signal.
pub trait Rectifier {
    /// Rectify a single sample of some incoming signal.
    fn rectify(sample: f32) -> f32;
}

impl Rectifier for PositiveHalfWave {
    #[inline]
    fn rectify(sample: f32) -> f32 {
        if sample < 0.0 { 0.0 } else { sample }
    }
}

impl Rectifier for NegativeHalfWave {
    #[inline]
    fn rectify(sample: f32) -> f32 {
        if sample > 0.0 { 0.0 } else { sample }
    }
}

impl Rectifier for FullWave {
    #[inline]
    fn rectify(sample: f32) -> f32 {
        sample.abs()
    }
}


/// A peak rectifier, generic over **FullWave**, **PositiveHalfWave** and **NegativeHalfWave**
/// rectification.
///
/// It produces a peak-following envelope when rectify is called over a signal of samples.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Peak<R=FullWave> {
    rectifier: PhantomData<R>,
}

impl Peak<FullWave> {
    /// A full-wave peak rectifier.
    pub fn full_wave() -> Peak<FullWave> {
        Peak {
            rectifier: PhantomData,
        }
    }
}

impl Peak<PositiveHalfWave> {
    /// A positive half-wave peak rectifier.
    pub fn positive_half_wave() -> Peak<PositiveHalfWave> {
        Peak {
            rectifier: PhantomData,
        }
    }
}

impl Peak<NegativeHalfWave> {
    /// A negative half-wave peak rectifier.
    pub fn negative_half_wave() -> Peak<NegativeHalfWave> {
        Peak {
            rectifier: PhantomData,
        }
    }
}

impl<R> Peak<R> {
    /// Return the rectified sample.
    #[inline]
    pub fn rectify(sample: f32) -> f32 where R: Rectifier {
        R::rectify(sample)
    }
}
