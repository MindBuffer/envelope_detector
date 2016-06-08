//! A generic interface over the two kinds of detection modes currently available to the
//! **EnvelopeDetector**.
//!
//! See the [**Mode**](./trait.Mode) trait.

use peak::{self, Peak};
use rms::Rms;
use sample::{Frame, Sample};


/// The mode used to detect the envelope of a signal.
pub trait Mode<F>
    where F: Frame,
{
    /// Update state that is unique to the **Mode**.
    fn next_frame(&mut self, frame: F) -> F;
}

impl<F, R> Mode<F> for Peak<R>
    where R: peak::Rectifier<F>,
          F: Frame,
{
    fn next_frame(&mut self, frame: F) -> F {
        Peak::<R>::rectify(frame)
    }
}

impl<F> Mode<F> for Rms<F>
    where F: Frame,
{
    fn next_frame(&mut self, frame: F) -> F {
        self.next(frame).map(|s| s.to_sample::<F::Sample>())
    }
}
