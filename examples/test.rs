//! Read the envelope from the input stream buffer (and pass the input buffer straight to the output).

extern crate dsp;
extern crate envelope_detector;
extern crate time_calc as time;

use envelope_detector::MultiChannelEnvelopeDetector;


use dsp::{CallbackFlags, CallbackResult, Settings, SoundStream, StreamParams};

fn main() {

    const CHANNELS: u16 = 2;
    const SAMPLE_HZ: f64 = 44_100.0;
    const WINDOW_SIZE_MS: f64 = 10.0;
    const ATTACK_MS: f64 = 1.0;
    const RELEASE_MS: f64 = 1.0;

    let window = time::Ms(WINDOW_SIZE_MS).samples(SAMPLE_HZ) as usize;
    let attack = time::Ms(ATTACK_MS).samples(SAMPLE_HZ) as f32;
    let release = time::Ms(RELEASE_MS).samples(SAMPLE_HZ) as f32;
    let channels = CHANNELS as usize;
    let mut envelope_detector = MultiChannelEnvelopeDetector::rms(window, attack, release, channels);

    // Callback used to construct the duplex sound stream.
    let callback = Box::new(move |input: &[f32], in_settings: Settings,
                                  output: &mut[f32], _out_settings: Settings,
                                  _dt: f64,
                                  _: CallbackFlags| {

        let n_frames = in_settings.frames as usize;
        let n_channels = in_settings.channels as usize;
        let sample_hz = in_settings.sample_hz as f64;
        let window_frames = time::Ms(WINDOW_SIZE_MS).samples(sample_hz) as usize;
        let attack = time::Ms(ATTACK_MS).samples(SAMPLE_HZ) as f32;
        let release = time::Ms(RELEASE_MS).samples(SAMPLE_HZ) as f32;

        envelope_detector.set_channels(n_channels);
        envelope_detector.set_window_frames(window_frames);
        envelope_detector.set_attack_frames(attack);
        envelope_detector.set_release_frames(release);

        let mut env = Vec::new();
        let mut idx = 0;
        for _ in 0..n_frames {
            env.clear();
            for j in 0..n_channels {
                let sample = input[idx];
                let env_sample = envelope_detector.next(j, sample);
                env.push(env_sample);
                idx += 1;
            }
            println!("\nsamples:  {:?}", &input[idx-n_channels..idx]);
            println!("envelope: {:?}", &env);
        }

        // Write the input to the output for fun.
        for (out_sample, in_sample) in output.iter_mut().zip(input.iter()) {
            *out_sample = *in_sample;
        }

        CallbackResult::Continue
    });

    // Construct parameters for a duplex stream and the stream itself.
    let params = StreamParams::new().channels(CHANNELS as i32);
    let stream = SoundStream::new()
        .sample_hz(SAMPLE_HZ)
        .frames_per_buffer(128)
        .duplex(params, params)
        .run_callback(callback)
        .unwrap();

    // Wait for our stream to finish.
    while let Ok(true) = stream.is_active() {
        ::std::thread::sleep_ms(16);
    }

}

