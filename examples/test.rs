//! Read the envelope from the input stream buffer (and pass the input buffer straight to the output).

extern crate envelope_detector;
extern crate portaudio as pa;
extern crate time_calc as time;

use envelope_detector::MultiChannelEnvelopeDetector;

fn main() {
    run().unwrap()
}

fn run() -> Result<(), pa::Error> {

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
    let callback = move |pa::DuplexStreamCallbackArgs { in_buffer, out_buffer, frames, .. }| {

        let n_frames = frames as usize;
        let n_channels = CHANNELS as usize;
        let window_frames = time::Ms(WINDOW_SIZE_MS).samples(SAMPLE_HZ) as usize;
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
                let sample = in_buffer[idx];
                let env_sample = envelope_detector.next(j, sample);
                env.push(env_sample);
                idx += 1;
            }
            println!("\nsamples:  {:?}", &in_buffer[idx-n_channels..idx]);
            println!("envelope: {:?}", &env);
        }

        // Write the input to the output for fun.
        for (out_sample, in_sample) in out_buffer.iter_mut().zip(in_buffer.iter()) {
            *out_sample = *in_sample;
        }

        pa::Continue
    };

    // Construct parameters for a duplex stream and the stream itself.
    const FRAMES: u32 = 128;
    let pa = try!(pa::PortAudio::new());
    let chan = CHANNELS as i32;
    let settings = try!(pa.default_duplex_stream_settings(chan, chan, SAMPLE_HZ, FRAMES));
    let mut stream = try!(pa.open_non_blocking_stream(settings, callback));
    try!(stream.start());

    // Wait for our stream to finish.
    while let true = try!(stream.is_active()) {
        ::std::thread::sleep_ms(16);
    }

    Ok(())
}

