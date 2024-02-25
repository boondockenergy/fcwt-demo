
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Stream;

pub struct Handle(Stream);

pub fn start_input() {
    let host = cpal::default_host();

    log::info!("Starting Input");
    for dev in host.input_devices().unwrap() {
        log::info!("Input device {}", dev.name().unwrap());
    }
}

#[cfg(target_arch = "wasm32")]
pub fn beep() -> Handle {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config().unwrap();

    log::info!("Output device {}", device.name().unwrap());

    for dev in host.input_devices().unwrap() {
        log::info!("Input device {}", dev.name().unwrap());
    }

    Handle(match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()),
        _ => panic!("unsupported sample format"),
    })
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Stream
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * 440.0 * 2.0 * 3.141592 / sample_rate).sin()
    };

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _| {
                log::info!("audio data..");
                write_data(data, channels, &mut next_value)
            },
            |e| {
                log::info!("Audio stream error");
            },
            None,
        )
        .unwrap();
    log::info!("Starting stream");
    stream.play().unwrap();
    stream
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: cpal::Sample + cpal::FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let sample = next_sample();
        let value = T::from_sample::<f32>(sample);
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}