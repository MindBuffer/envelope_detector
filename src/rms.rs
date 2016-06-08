//! Root mean square calculation over a signal.
//!
//! The primary type of interest in this module is the [**Rms**](./struct.Rms).

use sample::{FloatSample, Frame, Sample};
use std;


/// Iteratively extracts the RMS (root mean square) envelope from a window over a signal of
/// sample `Frame`s.
#[derive(Clone)]
pub struct Rms<F>
    where F: Frame,
{
    /// The type of `Frame`s for which the RMS will be calculated.
    frame: std::marker::PhantomData<F>,
    /// The ringbuffer of frame sample squares (i.e. `sample * sample`) used to calculate the RMS
    /// per sample.
    ///
    /// When a new sample is received, the **Rms** pops the front sample_square and adds the new
    /// sample_square to the back.
    window: std::collections::VecDeque<F::Float>,
    /// The sum total of all sample_squares currently within the **Rms**'s `window` ring buffer.
    sum: F::Float,
}

impl<F> std::fmt::Debug for Rms<F>
    where F: Frame,
          F::Float: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Rms {{ frame: {:?}, window: {:?}, sum: {:?} }}",
               &self.frame, &self.window, &self.sum)
    }
}


impl<F> Rms<F>
    where F: Frame,
{

    /// Construct a new **Rms**.
    pub fn new(n_window_frames: usize) -> Self {
        Rms {
            frame: std::marker::PhantomData,
            window: (0..n_window_frames).map(|_| Frame::equilibrium()).collect(),
            sum: Frame::equilibrium(),
        }
    }

    /// Zeroes the sum and the buffer of the `window`.
    pub fn reset(&mut self) {
        for sample_square in &mut self.window {
            *sample_square = Frame::equilibrium();
        }
        self.sum = Frame::equilibrium();
    }

    /// Set the size of the `window` as a number of frames.
    ///
    /// If the current window length is longer than the given length, the difference will be popped
    /// from the front of the `window` while adjusting the `sum` accordingly.
    ///
    /// If the current window length is shorter than the given length, the difference will be
    /// pushed to the front of the `window` using frames at signal equilibrium.
    ///
    /// If the length already is already correct, no re-sizing occurs.
    pub fn set_window_frames(&mut self, n_window_frames: usize) {
        let len = self.window.len();
        if len == n_window_frames {
            return;

        // If our window is too long, truncate it from the front (removing the olest frames).
        } else if len > n_window_frames {
            let diff = len - n_window_frames;
            for _ in 0..diff {
                self.pop_front();
            }

        // If our window is too short, we'll zero-pad the front of the ringbuffer (this way, the
        // padded zeroes will be the first to be removed).
        } else if len < n_window_frames {
            let diff = n_window_frames - len;
            for _ in 0..diff {
                self.window.push_front(Frame::equilibrium());
            }
        }
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
    pub fn next(&mut self, new_frame: F) -> F::Float {
        // If our **Window** has no length, there's nothing to calculate.
        if self.window.len() == 0 {
            return Frame::equilibrium();
        }
        self.pop_front();
        self.push_back(new_frame.to_float_frame());
        self.calc_rms()
    }

    /// Remove the front frame and subtract it from the `sum` frame.
    fn pop_front(&mut self) {
        let removed_sample_square = self.window.pop_front().unwrap();
        self.sum = self.sum.zip_map(removed_sample_square, |s, r| {
            let diff = s - r;
            // Don't let floating point rounding errors put us below 0.0.
            if diff < Sample::equilibrium() { Sample::equilibrium() } else { diff }
        });
    }

    /// Determines the square of the given frame, pushes it back onto our buffer and adds it to
    /// the `sum`.
    fn push_back(&mut self, new_frame: F::Float) {
        // Push back the new frame_square and add it to the `sum`.
        let new_frame_square = new_frame.zip_map(new_frame, |a, b| a * b);
        self.window.push_back(new_frame_square);
        self.sum = self.sum.add_amp(new_frame_square);
    }

    /// Calculate the RMS for the **Window** in its current state and yield the result as the
    /// `Frame`s associated `Float` type.
    fn calc_rms(&self) -> F::Float {
        let num_frames_f = Sample::from_sample(self.window.len() as f32);
        self.sum.map(|s| (s / num_frames_f).sample_sqrt())
    }

}
