//! A collection of types and traits useful for high performance envelope detection over a signal.
//!
//! The primary types of interest are:
//!
//! - [**EnvelopeDetector**](./struct.EnvelopeDetector).
//! - [**Rms**](./rms.struct.Rms).
//! - [**Peak**](./peak.struct.Peak).

#![deny(missing_copy_implementations)]
#![deny(missing_docs)]

extern crate sample;

pub use mode::Mode;
pub use peak::Peak;
pub use rms::Rms;
pub use sample::{Frame, Sample};

pub mod mode;
pub mod peak;
pub mod rms;


/// Iteratively extracts the amplitude envelope from an audio signal based on three parameters:
///
/// - Attack time.
/// - Release time.
/// - Detection mode (Either Peak or RMS).
///
/// Supports processing any `sample::Frame`
#[derive(Copy, Clone, Debug)]
pub struct EnvelopeDetector<F, M>
    where F: Frame,
          M: Mode<F>,
{
    attack_gain: f32,
    release_gain: f32,
    last_env_frame: F,
    mode: M,
}

/// An `EnvelopeDetector` that tracks the signal envelope using RMS.
pub type RmsEnvelopeDetector<F> = EnvelopeDetector<F, Rms<F>>;
/// An `EnvelopeDetector` that tracks the full wave `Peak` envelope of a signal.
pub type PeakEnvelopeDetector<F> = EnvelopeDetector<F, Peak<peak::FullWave>>;


fn calc_gain(n_frames: f32) -> f32 {
    ::std::f32::consts::E.powf(-1.0 / n_frames)
}


impl<F> EnvelopeDetector<F, Rms<F>>
    where F: Frame,
{

    /// Construct a new **Rms** **EnvelopeDetector**.
    pub fn rms(rms_window_frames: usize, attack_frames: f32, release_frames: f32) -> Self {
        let rms = Rms::new(rms_window_frames);
        Self::new(rms, attack_frames, release_frames)
    }

    /// Set the duration of the **Rms** window in frames.
    pub fn set_window_frames(&mut self, n_window_frames: usize) {
        self.mode.set_window_frames(n_window_frames);
    }

}

impl<F> EnvelopeDetector<F, Peak<peak::FullWave>>
    where F: Frame,
{

    /// Construct a new **Mono** **Peak** **EnvelopeDetector**.
    pub fn peak(attack_frames: f32, release_frames: f32) -> Self {
        let peak = Peak::full_wave();
        Self::new(peak, attack_frames, release_frames)
    }

}

impl<F, M> EnvelopeDetector<F, M>
    where F: Frame,
          M: Mode<F>,
{

    fn new(mode: M, attack_frames: f32, release_frames: f32) -> Self {
        EnvelopeDetector {
            mode: mode,
            last_env_frame: F::equilibrium(),
            attack_gain: calc_gain(attack_frames),
            release_gain: calc_gain(release_frames),
        }
    }

    /// Set the **EnvelopeDetector**'s attack time as a number of frames.
    pub fn set_attack_frames(&mut self, frames: f32) {
        self.attack_gain = calc_gain(frames);
    }

    /// Set the **EnvelopeDetector**'s release time as a number of frames.
    pub fn set_release_frames(&mut self, frames: f32) {
        self.attack_gain = calc_gain(frames);
    }

    /// Given the next input signal frame, detect and return the next envelope frame.
    pub fn next(&mut self, frame: F) -> F {
        let EnvelopeDetector {
            attack_gain, release_gain, ref mut mode, ref mut last_env_frame,
        } = *self;

        let mode_frame = mode.next_frame(frame);
        let new_env_frame = last_env_frame.zip_map(mode_frame, |l, m| {
            let gain = if l < m { attack_gain } else { release_gain };
            let diff = l.add_amp(-m.to_signed_sample());
            m.add_amp(diff.mul_amp(gain.to_sample()).to_sample())
        });
        *last_env_frame = new_env_frame;
        new_env_frame
    }

}
