//! Read the envelope from the input stream buffer (and pass the input buffer straight to the output).

extern crate envelope_detector;
extern crate portaudio as pa;
extern crate sample;
extern crate time_calc as time;

use envelope_detector::EnvelopeDetector;

fn main() {
    run().unwrap()
}

fn run() -> Result<(), pa::Error> {

    const CHANNELS: usize = 2;
    const SAMPLE_HZ: f64 = 44_100.0;
    const WINDOW_SIZE_MS: f64 = 10.0;
    const ATTACK_MS: f64 = 1.0;
    const RELEASE_MS: f64 = 1.0;

    let window = time::Ms(WINDOW_SIZE_MS).samples(SAMPLE_HZ) as usize;
    let attack = time::Ms(ATTACK_MS).samples(SAMPLE_HZ) as f32;
    let release = time::Ms(RELEASE_MS).samples(SAMPLE_HZ) as f32;
    let mut envelope_detector = EnvelopeDetector::rms(window, attack, release);

    // Callback used to construct the duplex sound stream.
    let callback = move |pa::InputStreamCallbackArgs { buffer, .. }| {

        let window_frames = time::Ms(WINDOW_SIZE_MS).samples(SAMPLE_HZ) as usize;
        let attack = time::Ms(ATTACK_MS).samples(SAMPLE_HZ) as f32;
        let release = time::Ms(RELEASE_MS).samples(SAMPLE_HZ) as f32;

        envelope_detector.set_window_frames(window_frames);
        envelope_detector.set_attack_frames(attack);
        envelope_detector.set_release_frames(release);

        let in_buffer: &[[f32; CHANNELS]] = sample::slice::to_frame_slice(buffer).unwrap();

        for &frame in in_buffer {
            let env_frame = envelope_detector.next(frame);
            println!("frame: {:?}", frame);
            println!("env_frame: {:?}", env_frame);
        }

        pa::Continue
    };

    // Construct parameters for a duplex stream and the stream itself.
    const FRAMES: u32 = 128;
    let pa = try!(pa::PortAudio::new());
    let chan = CHANNELS as i32;
    let settings = try!(pa.default_input_stream_settings::<f32>(chan, SAMPLE_HZ, FRAMES));
    let mut stream = try!(pa.open_non_blocking_stream(settings, callback));
    try!(stream.start());

    // Wait for our stream to finish.
    while let true = try!(stream.is_active()) {
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    Ok(())
}
