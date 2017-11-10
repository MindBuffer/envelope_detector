//! Root mean square calculation over a signal.
//!
//! The primary type of interest in this module is the [**Rms**](./struct.Rms).

use sample::{ring_buffer, FloatSample, Frame, Sample};
use std;


/// Iteratively extracts the RMS (root mean square) envelope from a window over a signal of sample
/// `Frame`s.
#[derive(Clone)]
pub struct Rms<F, S>
    where F: Frame,
          S: ring_buffer::Slice<Element=F::Float>,
{
    /// The type of `Frame`s for which the RMS will be calculated.
    frame: std::marker::PhantomData<F>,
    /// The ringbuffer of frame sample squares (i.e. `sample * sample`) used to calculate the RMS
    /// per frame.
    ///
    /// When a new frame is received, the **Rms** pops the front sample_square and adds the new
    /// sample_square to the back.
    window: ring_buffer::Fixed<S>,
    /// The sum total of all sample_squares currently within the **Rms**'s `window` ring buffer.
    sum: F::Float,
}

impl<F, S> std::fmt::Debug for Rms<F, S>
    where F: Frame,
          F::Float: std::fmt::Debug,
          S: std::fmt::Debug + ring_buffer::Slice<Element=F::Float>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Rms {{ frame: {:?}, window: {:?}, sum: {:?} }}",
               &self.frame, &self.window, &self.sum)
    }
}


impl<F, S> Rms<F, S>
    where F: Frame,
          S: ring_buffer::Slice<Element=F::Float>,
{
    /// Construct a new **Rms** that uses the given ring buffer as its window.
    ///
    /// The window size of the **Rms** is equal to the length of the given ring buffer.
    pub fn new(ring_buffer: ring_buffer::Fixed<S>) -> Self {
        Rms {
            frame: std::marker::PhantomData,
            window: ring_buffer,
            sum: Frame::equilibrium(),
        }
    }

    /// Zeroes the sum and the buffer of the `window`.
    pub fn reset(&mut self)
    where
        S: ring_buffer::SliceMut,
    {
        for sample_square in self.window.iter_mut() {
            *sample_square = Frame::equilibrium();
        }
        self.sum = Frame::equilibrium();
    }

    /// The length of the window as a number of frames.
    #[inline]
    pub fn window_frames(&self) -> usize {
        self.window.len()
    }

    /// The next RMS given the new frame in the sequence.
    ///
    /// The **Rms** pops its front frame and adds the new frame to the back.
    ///
    /// The yielded RMS is the RMS of all frame squares in the `window` after the new frame is
    /// added.
    ///
    /// Returns `Frame::equilibrium` if the `window` is empty.
    #[inline]
    pub fn next(&mut self, new_frame: F) -> F::Float
    where
        S: ring_buffer::SliceMut,
    {
        // Determine the square of the new frame.
        let new_frame_square = new_frame.to_float_frame().map(|s| s * s);
        // Push back the new frame_square.
        let removed_frame_square = self.window.push(new_frame_square);
        // Add the new frame square and subtract the removed frame square.
        self.sum = self.sum
            .add_amp(new_frame_square)
            .zip_map(removed_frame_square, |s, r| {
                let diff = s - r;
                // Don't let floating point rounding errors put us below 0.0.
                if diff < Sample::equilibrium() { Sample::equilibrium() } else { diff }
            });
        self.calc_rms()
    }

    /// Calculate the RMS for the **Window** in its current state and yield the result as the
    /// `Frame`s associated `Float` type.
    fn calc_rms(&self) -> F::Float {
        let num_frames_f = Sample::from_sample(self.window.len() as f32);
        self.sum.map(|s| (s / num_frames_f).sample_sqrt())
    }

    /// Consumes the **Rms** and returns its inner ring buffer of squared frames along with a frame
    /// representing the sum of all frame squares contained within the ring buffer.
    pub fn into_parts(self) -> (ring_buffer::Fixed<S>, S::Element) {
        let Rms { window, sum, .. } = self;
        (window, sum)
    }
}
