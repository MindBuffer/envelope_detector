//! A generic interface over the two kinds of detection modes currently available to the
//! **EnvelopeDetector**.
//!
//! See the [**Mode**](./trait.Mode) trait.

use peak::{self, Peak};
use rms::Rms;


/// The mode used to detect the envelope of a signal.
pub trait Mode {
    /// Update state that is unique to the **Mode**.
    fn next_sample(&mut self, sample: f32) -> f32;
}


impl<R> Mode for Peak<R> where R: peak::Rectifier {
    fn next_sample(&mut self, sample: f32) -> f32 {
        Peak::<R>::rectify(sample)
    }
}


impl Mode for Rms {
    fn next_sample(&mut self, sample: f32) -> f32 {
        self.next(sample)
    }
}

