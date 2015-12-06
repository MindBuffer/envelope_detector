//! Root mean square calculation over a signal.
//!
//! The primary type of interest in this module is the [**Rms**](./struct.Rms).

use std::collections::VecDeque;


/// Iteratively extracts the RMS (root mean square) envelope from a window over a signal of
/// samples.
#[derive(Clone, Debug)]
pub struct Rms {
    /// The ringbuffer of sample squares (i.e. `sample.powf(2.0)`) used to calculate the RMS per
    /// sample.
    ///
    /// When a new sample is received, the **Rms** pops the front sample_square and adds the new
    /// sample_square to the back.
    window: VecDeque<f32>,
    /// The sum total of all sample_squares currently within the **Rms**'s `window` ring buffer.
    sum: f32,
}


impl Rms {

    /// Construct a new **Rms**.
    pub fn new(n_window_frames: usize) -> Self {
        Rms {
            window: (0..n_window_frames).map(|_| 0.0).collect(),
            sum: 0.0,
        }
    }

    /// Zeroes the sum and the buffer of the `window`.
    pub fn reset(&mut self) {
        for sample_square in &mut self.window {
            *sample_square = 0.0;
        }
        self.sum = 0.0;
    }

    /// Set the size of the `window` as a number of frames.
    ///
    /// If the current window length is longer than the given length, the difference will be popped
    /// from the front of the `window` while adjusting the `sum` accordingly.
    ///
    /// If the current window length is shorter than the given length, the difference will be
    /// pushed to the front of the `window` using an averaged frame in order to retain the current
    /// RMS.
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

        // If our window is too short, extend it using fake frames.
        //
        // We'll generate the fake samples as the current average to avoid affecting the `window`'s
        // RMS output as much as possible.
        } else if len < n_window_frames {
            let diff = n_window_frames - len;
            let avg = self.sum / self.window.len() as f32;
            for _ in 0..diff {
                // Push the new fake samples onto the front so they are the first to be removed.
                self.window.push_front(avg);
                self.sum += avg;
            }
        }
    }

    /// The length of the window as a number of frames.
    pub fn window_frames(&self) -> usize {
        self.window.len()
    }

    /// The next RMS given the new sample in the sequence.
    ///
    /// The **Rms** pops its front sample and adds the new sample to the back.
    ///
    /// The yielded RMS is the RMS of all sample squares in the `window` after the new sample is
    /// added.
    ///
    /// Returns `0.0` if the `window` is empty.
    pub fn next(&mut self, new_sample: f32) -> f32 {
        // If our **Window** has no length, there's nothing to calculate.
        if self.window.len() == 0 {
            return 0.0;
        }
        self.pop_front();
        self.push_back(new_sample);
        self.calc_rms()
    }

    /// Remove the front sample and subtract it from the `sum`.
    fn pop_front(&mut self) {
        let removed_sample_square = self.window.pop_front().unwrap();
        self.sum -= removed_sample_square;
        // Don't let floating point rounding errors put us below 0.0.
        if self.sum < 0.0 {
            self.sum = 0.0;
        }
    }

    /// Determines the square of the given sample, pushes it back onto our buffer and adds it to
    /// the `sum`.
    fn push_back(&mut self, new_sample: f32) {
        // Push back the new sample_square and add it to the `sum`.
        let new_sample_square = new_sample.powf(2.0);
        self.window.push_back(new_sample_square);
        self.sum += new_sample_square;
    }

    /// Calculate the RMS for the **Window** in its current state.
    fn calc_rms(&self) -> f32 {
        (self.sum / self.window.len() as f32).sqrt()
    }

}

