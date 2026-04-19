//! Process (stereo) input and play the result (in stereo).

use std::f32::consts::{PI, TAU};
use std::sync::Mutex;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample, Stream};
use fundsp::hacker32::*;

use crossbeam_channel::{bounded, Receiver, Sender};
use tauri::Manager;

#[derive(Clone)]
pub struct InputNode {
    receiver: Receiver<(f32, f32)>,
}

impl InputNode {
    pub fn new(receiver: Receiver<(f32, f32)>) -> Self {
        InputNode { receiver }
    }
}

impl AudioNode for InputNode {
    const ID: u64 = 87;
    type Inputs = U0;
    type Outputs = U2;

    #[inline]
    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let (left, right) = self.receiver.try_recv().unwrap_or((0.0, 0.0));
        [left, right].into()
    }
}

pub async fn start_audio(handle: tauri::AppHandle, r: Receiver<i16>) {
    // Sender / receiver for left and right channels (stereo mic).
    let (sender, receiver) = bounded(8192);

    let host = cpal::default_host();
    // Start input.
    let in_devices = host.input_devices();
    let mut in_device = host.default_input_device().unwrap();

    let defaults = handle.state::<Mutex<[String; 2]>>();
    let mut defaults = defaults.lock().unwrap();

    in_devices.expect("").for_each(|device| {
        println!("{}", device.name().unwrap());
        if device.name().expect("") == defaults[0] {
            in_device = device;
        }
    });

    let in_config = in_device.default_input_config().unwrap();
    let mut input_stream: Stream;
    match in_config.sample_format() {
        cpal::SampleFormat::F32 => input_stream = run_in::<f32>(&in_device, &in_config.into(), sender.clone()),
        cpal::SampleFormat::I16 => input_stream = run_in::<i16>(&in_device, &in_config.into(), sender.clone()),
        cpal::SampleFormat::U16 => input_stream = run_in::<u16>(&in_device, &in_config.into(), sender.clone()),
        format => {eprintln!("Unsupported sample format: {}", format); return;},
    }

    // Start output.
    let out_devices = host.output_devices();
    let mut out_device = host.default_output_device().unwrap();

    out_devices.expect("").for_each(|device| {
        println!("{}", device.name().unwrap());
        if device.name().expect("") == defaults[1] {
            out_device = device;
        }
    });

    let out_config = out_device.default_output_config().unwrap();
    let mut output_stream: Stream;
    match out_config.sample_format() {
        cpal::SampleFormat::F32 => output_stream = run_out::<f32>(&out_device, &out_config.into(), &receiver, handle.clone()),
        cpal::SampleFormat::I16 => output_stream = run_out::<i16>(&out_device, &out_config.into(), &receiver, handle.clone()),
        cpal::SampleFormat::U16 => output_stream = run_out::<u16>(&out_device, &out_config.into(), &receiver, handle.clone()),
        format => {eprintln!("Unsupported sample format: {}", format); return;},
    }
    println!("Processing stereo input to stereo output.");
    drop(defaults);
    
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let t = r.recv();
        println!("{}", t.expect("a"));

        println!("reset");
        drop(output_stream);
        drop(input_stream);

        let defaults = handle.state::<Mutex<[String; 2]>>();
        let mut defaults = defaults.lock().unwrap();

        host.input_devices().expect("").for_each(|device| {
            if device.name().expect("") == defaults[0] {
                in_device = device;
            }
        });

        let in_config = in_device.default_input_config().unwrap();
        match in_config.sample_format() {
            cpal::SampleFormat::F32 => input_stream = run_in::<f32>(&in_device, &in_config.into(), sender.clone()),
            cpal::SampleFormat::I16 => input_stream = run_in::<i16>(&in_device, &in_config.into(), sender.clone()),
            cpal::SampleFormat::U16 => input_stream = run_in::<u16>(&in_device, &in_config.into(), sender.clone()),
            format => {eprintln!("Unsupported sample format: {}", format); return;},
        }

        host.output_devices().expect("").for_each(|device| {
            if device.name().expect("") == defaults[1] {
                out_device = device;
            }
        });

        let out_config = out_device.default_output_config().unwrap();   
        match out_config.sample_format() {
            cpal::SampleFormat::F32 => output_stream = run_out::<f32>(&out_device, &out_config.into(), &receiver, handle.clone()),
            cpal::SampleFormat::I16 => output_stream = run_out::<i16>(&out_device, &out_config.into(), &receiver, handle.clone()),
            cpal::SampleFormat::U16 => output_stream = run_out::<u16>(&out_device, &out_config.into(), &receiver, handle.clone()),
            format => {eprintln!("Unsupported sample format: {}", format); return;},
        }

        drop(defaults);
    }
}

fn run_in<T>(device: &cpal::Device, config: &cpal::StreamConfig, sender: Sender<(f32, f32)>) -> Stream
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let channels = config.channels as usize;
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let stream = device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| read_data(data, channels, sender.clone()),
        err_fn,
        None,
    );
    if let Ok(stream) = stream {
        if let Ok(()) = stream.play() {
            return stream;
            std::mem::forget(stream);
        }
        else{
            return stream;
        }
    }
    else {
        return stream.unwrap();
    }
    println!("Input stream built.");
}

fn read_data<T>(input: &[T], channels: usize, sender: Sender<(f32, f32)>)
where
    T: SizedSample,
    f32: FromSample<T>,
{
    for frame in input.chunks(channels) {
        let mut left = 0.0;
        let mut right = 0.0;
        for (channel, sample) in frame.iter().enumerate() {
            if channel & 1 == 0 {
                left = sample.to_sample::<f32>();
            } else {
                right = sample.to_sample::<f32>();
            }
        }
        if let Ok(()) = sender.try_send((left, right)) {}
    }
}

fn run_out<T>(device: &cpal::Device, config: &cpal::StreamConfig, receiver: &Receiver<(f32, f32)>, handle: tauri::AppHandle) -> Stream
where
    T: SizedSample + FromSample<f32>,
{
    let rec_sample_rate = config.sample_rate.0;

    let channels = config.channels as usize;

    let values = handle.state::<Mutex<[f32; 3]>>();
    let mut values = values.lock().unwrap();

    let freq = values[0];
    let flang = values[1];
    let bandpass = values[2];

    let input = An(InputNode::new(receiver.to_owned()));
    
    let chorus = chorus(0, 0.0, 0.03, 0.2) | chorus(1, 0.0, 0.03, 0.2);
    // let res = dc(1.05) >> resample(input);
    println!("{}", freq);

    let pitch_shift = freq;
    const FRAME_SIZE: usize = 512 * 2;
    let mut input_phases = [0.0; FRAME_SIZE];
    let mut incoming_frequencies = [0.0; FRAME_SIZE];
    let mut outgoing_phases = [0.0; FRAME_SIZE];
    let bin_width = rec_sample_rate as f32 / FRAME_SIZE as f32;

    const WINDOWS: usize = 4;
    let dt = FRAME_SIZE as f32 / rec_sample_rate as f32 / WINDOWS as f32;

    let pitch_shift = resynth::<U2, U2, _>(FRAME_SIZE, move |fft: &mut FftWindow| {
            for i in 0..(fft.bins() - 1) {
                let freq = fft.frequency(i);
                // phase [-pi, pi]
                let (amplitude, phase) = fft.at(0, i).to_polar();
                // [0, tau)
                let phase_delta = (phase - input_phases[i] + TAU) % TAU;
                // [0, tau)
                let expected_phase_d = freq * dt * TAU % TAU;
                // [-pi, pi)
                let phase_error = (phase_delta - expected_phase_d) % PI;

                // phase offset to frequency
                let freq_deviation = phase_error / TAU / dt;
                let bin_deviation = freq_deviation / bin_width;

                incoming_frequencies[i] = i as f32 + bin_deviation;

                input_phases[i] = phase;

                // calc output
                let newbin = (i as f32 * pitch_shift).round() as usize;

                if newbin > 0 && newbin < fft.bins() {
                    let bin_deviation = incoming_frequencies[i] * pitch_shift - newbin as f32;
                    let freq = bin_deviation * bin_width + fft.frequency(newbin);
                    let phase_diff = freq * dt * TAU;
                    let out_phase = (outgoing_phases[newbin] + phase_diff) % TAU;

                    fft.set(0, newbin, Complex32::from_polar(amplitude, out_phase));
                    fft.set(1, newbin, Complex32::from_polar(amplitude, out_phase));
                    outgoing_phases[newbin] = out_phase;
                }
            }
        });

    let enabled = handle.state::<Mutex<[bool; 3]>>();
    let mut enabled = enabled.lock().unwrap();
    
    let mut post_pass = Net::wrap(Box::new(input));

    let pass = bandpass_hz(bandpass, 2.0) | bandpass_hz(bandpass, 2.0);
    if (enabled[2]) {
        post_pass = post_pass >> pass;
    }

    if (enabled[0]) {
        post_pass = post_pass >> pitch_shift;
    }

    let flanger = flanger(flang, 0.05, 0.10, |t| lerp11(0.01, 0.02, sin_hz(0.1, t))) | flanger(flang, 0.05, 0.10, |t| lerp11(0.01, 0.02, sin_hz(0.1, t)));
    if (enabled[1]) {
        post_pass = post_pass >> flanger;
    }

    // Here is the final input-to-output processing chain.
    let graph = post_pass;
    let mut graph = BlockRateAdapter::new(Box::new(graph));
    graph.set_sample_rate(config.sample_rate.0 as f64);

    let mut next_value = move || graph.get_stereo();

    let err_fn = |err| eprintln!("An error occurred on stream: {}", err);
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
        None,
    );

    if let Ok(stream) = stream {
        if let Ok(()) = stream.play() {
            return stream;
            std::mem::forget(stream);
        }
        else{
            return stream;
        }
    }
    else {
        return stream.unwrap();
    }
    println!("Output stream built.");
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> (f32, f32))
where
    T: SizedSample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let sample = next_sample();
        let left = T::from_sample(sample.0);
        let right = T::from_sample(sample.1);

        for (channel, sample) in frame.iter_mut().enumerate() {
            if channel & 1 == 0 {
                *sample = left;
            } else {
                *sample = right;
            }
        }
    }
}
