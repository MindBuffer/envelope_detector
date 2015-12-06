//! A collection of types and traits useful for high performance envelope detection over a signal.
//!
//! The primary types of interest are:
//!
//! - [**EnvelopeDetector**](./struct.EnvelopeDetector).
//! - [**Rms**](./rms.struct.Rms).
//! - [**Peak**](./peak.struct.Peak).

#![deny(missing_copy_implementations)]
#![deny(missing_docs)]

pub use mode::Mode;
pub use peak::Peak;
pub use rms::Rms;

pub mod mode;
pub mod peak;
pub mod rms;


/// Iteratively extracts the amplitude envelope from an audio signal based on three parameters:
///
/// - Attack time.
/// - Release time.
/// - Detection mode (Either Peak or RMS).
///
/// The **EnvelopeDetector** supports both [**Mono**](./struct.Mono) and
/// [**MultiChannel**](./struct.MultiChannel) modes.
#[derive(Copy, Clone, Debug)]
pub struct EnvelopeDetector<C> {
    channel_mode: C,
    attack_gain: f32,
    release_gain: f32,
}

/// A channel mode used by the **EnvelopeDetector** when envelope detection is only needed on a
/// single channel.
#[derive(Copy, Clone, Debug)]
pub struct Mono<D> {
    detection_mode: D,
    last_env_sample: f32,
}

/// A channel mode that allows the **EnvelopeDetector** to handle any number of audio channels.
#[derive(Clone, Debug)]
pub struct MultiChannel<D> {
    channels: Vec<Mono<D>>,
}


/// A single channel **EnvelopeDetector** generic over its detection mode.
pub type MonoEnvelopeDetector<D> = EnvelopeDetector<Mono<D>>;

/// A multi-channel **EnvelopeDetector** generic over its detection mode.
pub type MultiChannelEnvelopeDetector<D> = EnvelopeDetector<MultiChannel<D>>;


fn calc_gain(frames: f32) -> f32 {
    ::std::f32::consts::E.powf(-1.0 / frames)
}


impl<C> EnvelopeDetector<C> {

    fn new(channel_mode: C, attack_frames: f32, release_frames: f32) -> Self {
        EnvelopeDetector {
            channel_mode: channel_mode,
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

}


impl<D> Mono<D> {

    /// Construct a new **Mono** from its detection mode.
    fn new(detection_mode: D) -> Self {
        Mono {
            last_env_sample: 0.0,
            detection_mode: detection_mode,
        }
    }

    /// Given the next input signal sample, detect and return the next envelope sample.
    #[inline]
    pub fn next(&mut self, sample: f32, attack_gain: f32, release_gain: f32) -> f32
        where D: Mode,
    {
        let mode_sample = self.detection_mode.next_sample(sample);
        let last_env_sample = self.last_env_sample;
        let gain = if last_env_sample < mode_sample { attack_gain } else { release_gain };
        let new_env_sample = mode_sample + gain * (last_env_sample - mode_sample);
        self.last_env_sample = new_env_sample;
        new_env_sample
    }

}

impl Mono<Rms> {

    /// Construct a new **Rms** **Mono**.
    pub fn rms(rms_window_frames: usize) -> Mono<Rms> {
        let rms = Rms::new(rms_window_frames);
        Self::new(rms)
    }

    /// Set the duration of the **Rms** window in frames.
    pub fn set_window_frames(&mut self, n_window_frames: usize) {
        self.detection_mode.set_window_frames(n_window_frames);
    }

}

impl Mono<Peak> {

    /// Construct a new **Rms** **Mono**.
    pub fn peak() -> Mono<Peak> {
        let peak = Peak::full_wave();
        Mono::new(peak)
    }

}


impl MonoEnvelopeDetector<Rms> {

    /// Construct a new **Mono** **Rms** **EnvelopeDetector**.
    pub fn rms(rms_window_frames: usize,
               attack_frames: f32,
               release_frames: f32) -> MonoEnvelopeDetector<Rms>
    {
        let mono = Mono::rms(rms_window_frames);
        Self::new(mono, attack_frames, release_frames)
    }

    /// Set the duration of the **Rms** window in frames.
    pub fn set_window_frames(&mut self, n_window_frames: usize) {
        self.channel_mode.set_window_frames(n_window_frames);
    }

}

impl MonoEnvelopeDetector<Peak> {

    /// Construct a new **Mono** **Peak** **EnvelopeDetector**.
    pub fn peak(attack_frames: f32,
                release_frames: f32) -> MonoEnvelopeDetector<Peak>
    {
        let mono = Mono::peak();
        Self::new(mono, attack_frames, release_frames)
    }

}

impl<D> MonoEnvelopeDetector<D> {

    /// Given the next input signal sample, detect and return the next envelope sample.
    #[inline]
    pub fn next(&mut self, sample: f32) -> f32 where D: Mode {
        self.channel_mode.next(sample, self.attack_gain, self.release_gain)
    }

}


impl<D> MultiChannel<D> {

    /// Resize the `channels` `Vec` to the given size.
    pub fn resize(&mut self, n_channels: usize) where D: Clone {
        let len = self.channels.len();
        if len == n_channels {
            return;
        } else if len > n_channels {
            self.channels.truncate(n_channels);
        } else if len < n_channels {
            let clone = self.channels.last().unwrap().clone();
            let extension = (0..n_channels).map(|_| clone.clone());
            self.channels.extend(extension);
        }
    }

}


impl MultiChannelEnvelopeDetector<Rms> {

    /// Construct a new **MultiChannel** **Rms** **EnvelopeDetector**.
    pub fn rms(rms_window_frames: usize,
               attack_frames: f32,
               release_frames: f32,
               n_channels: usize) -> MultiChannelEnvelopeDetector<Rms>
    {
        assert!(n_channels > 0, "We can't do anything with no channels");
        let multi_channel = MultiChannel {
            channels: (0..n_channels).map(|_| Mono::rms(rms_window_frames)).collect(),
        };
        EnvelopeDetector::new(multi_channel, attack_frames, release_frames)
    }

    /// Set the duration of the **Rms** window in frames.
    pub fn set_window_frames(&mut self, n_window_frames: usize) {
        for channel in &mut self.channel_mode.channels {
            channel.set_window_frames(n_window_frames);
        }
    }

}

impl MultiChannelEnvelopeDetector<Peak> {

    /// Construct a new **MultiChannel** **Peak** **EnvelopeDetector**.
    pub fn peak(attack_frames: f32,
                release_frames: f32,
                n_channels: usize) -> MultiChannelEnvelopeDetector<Peak>
    {
        assert!(n_channels > 0, "We can't do anything with no channels");
        let multi_channel = MultiChannel {
            channels: (0..n_channels).map(|_| Mono::peak()).collect(),
        };
        EnvelopeDetector::new(multi_channel, attack_frames, release_frames)
    }

}

impl<D> MultiChannelEnvelopeDetector<D> {

    /// Set the number of channels for the **MultiChannelEnvelopeDetector**.
    pub fn set_channels(&mut self, n_channels: usize) where D: Clone {
        assert!(n_channels > 0, "We can't do anything with no channels");
        self.channel_mode.resize(n_channels)
    }

    /// Given the next input signal sample for the channel at the given index, detect and return
    /// the next envelope sample for that channel.
    ///
    /// **Panics** if the given channel_idx is out of range.
    #[inline]
    pub fn next(&mut self, channel_idx: usize, sample: f32) -> f32 where D: Mode {
        self.channel_mode.channels[channel_idx].next(sample, self.attack_gain, self.release_gain)
    }

}

