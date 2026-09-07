//! Audio output: pull the APU's samples out of the shared buffer and play them
//! through the host sound device with cpal. Compiled only with the `audio`
//! feature (the default).

use chlorosis_core::{AudioBuffer, SAMPLE_RATE};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Stream;

/// Start streaming `buffer` to the default output device. Returns the live
/// stream, which must be kept alive for playback to continue; `None` (with a
/// logged reason) if no device is available or the stream cannot start, in
/// which case the emulator simply runs without sound.
pub fn start(buffer: AudioBuffer) -> Option<Stream> {
    let host = cpal::default_host();
    let Some(device) = host.default_output_device() else {
        eprintln!("No audio output device; running without sound");
        return None;
    };

    let config = cpal::StreamConfig {
        channels: 2,
        sample_rate: cpal::SampleRate(SAMPLE_RATE),
        buffer_size: cpal::BufferSize::Default,
    };

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Pop interleaved samples; underflow (the emulator falling behind or
            // paused) plays silence rather than stuttering on stale audio.
            match buffer.lock() {
                Ok(mut buf) => {
                    for sample in data.iter_mut() {
                        *sample = buf.pop_front().unwrap_or(0.0);
                    }
                }
                Err(_) => data.fill(0.0),
            }
        },
        |err| eprintln!("Audio stream error: {err}"),
        None,
    );

    match stream {
        Ok(stream) => match stream.play() {
            Ok(()) => Some(stream),
            Err(e) => {
                eprintln!("Could not start audio playback: {e}; running without sound");
                None
            }
        },
        Err(e) => {
            eprintln!("Could not open audio stream: {e}; running without sound");
            None
        }
    }
}
