//! Peak envelope detection over a signal.
//!
//! The primary type of interest in this module is the [**Peak**](./struct.Peak) type, generic
//! over any [**Rectifier**](./trait.Rectifier).

use sample::{Frame, Sample};
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
pub trait Rectifier<F>
    where F: Frame,
{
    /// Rectify a single sample of some incoming signal.
    fn rectify(frame: F) -> F;
}

impl<F> Rectifier<F> for PositiveHalfWave
    where F: Frame,
{
    #[inline]
    fn rectify(frame: F) -> F {
        frame.map(|s| if s < Sample::equilibrium() { Sample::equilibrium() } else { s })
    }
}

impl<F> Rectifier<F> for NegativeHalfWave
    where F: Frame,
{
    #[inline]
    fn rectify(frame: F) -> F {
        frame.map(|s| if s > Sample::equilibrium() { Sample::equilibrium() } else { s })
    }
}

impl<F> Rectifier<F> for FullWave
    where F: Frame,
{
    #[inline]
    fn rectify(frame: F) -> F {
        frame.map(|s| {
            let signed = s.to_signed_sample();
            if signed < Sample::equilibrium() { -signed } else { signed }
                .to_sample()
        })
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
    pub fn rectify<F>(frame: F) -> F
        where R: Rectifier<F>,
              F: Frame,
    {
        R::rectify(frame)
    }
}
