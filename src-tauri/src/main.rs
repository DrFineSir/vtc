#![warn(clippy::all)]
#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

use std::ops::{Deref, DerefMut};

use enigo::*;
use parking_lot::Mutex;

extern crate anyhow;
extern crate clap;
extern crate cpal;
extern crate ringbuf;

use anyhow::Context;
use clap::arg;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::RingBuffer;

struct Mouse(Enigo);

unsafe impl Sync for Mouse {}

impl Deref for Mouse {
  type Target = Enigo;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for Mouse {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

#[derive(Debug)]
struct Opt {
  #[cfg(all(
  any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"),
  feature = "jack"
  ))]
  jack: bool,

  latency: f32,
  input_device: String,
  output_device: String,
}

impl Opt {
  fn from_args() -> anyhow::Result<Self> {
    let app = clap::Command::new("feedback")
        .arg(arg!(
            -l --latency [DELAY_MS] "Specify the delay between input and output [default: 150]"))
        .arg(arg!([IN] "The input audio device to use"))
        .arg(arg!([OUT] "The output audio device to use"));

    #[cfg(all(
    any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"),
    feature = "jack"
    ))]
        let app = app.arg(arg!(-j --jack "Use the JACK host"));
    let matches = app.get_matches();
    let latency: f32 = matches
        .value_of("latency")
        .unwrap_or("150")
        .parse()
        .context("parsing latency option")?;
    let input_device = matches.value_of("IN").unwrap_or("default").to_string();
    let output_device = matches.value_of("OUT").unwrap_or("default").to_string();

    #[cfg(all(
    any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"),
    feature = "jack"
    ))]
    return Ok(Opt {
      jack: matches.is_present("jack"),
      latency,
      input_device,
      output_device,
    });

    #[cfg(any(
    not(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd")),
    not(feature = "jack")
    ))]
    Ok(Opt {
      latency,
      input_device,
      output_device,
    })
  }
}

#[tauri::command]
fn mouse_click(mouse: tauri::State<Mutex<Mouse>>) {
  mouse.lock().mouse_click(MouseButton::Left)
}

fn main() -> anyhow::Result<()> {
  tauri::Builder::default()
      .manage(Mutex::new(Mouse(Enigo::new())))
      .invoke_handler(tauri::generate_handler![mouse_click])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");

  let opt = Opt::from_args()?;

  #[cfg(any(
  not(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd")),
  not(feature = "jack")
  ))]
      let host = cpal::default_host();

  // Find devices.
  let input_device = if opt.input_device == "default" {
    host.default_input_device()
  } else {
    host.input_devices()?
        .find(|x| x.name().map(|y| y == opt.input_device).unwrap_or(false))
  }
      .expect("failed to find input device");

  let output_device = if opt.output_device == "default" {
    host.default_output_device()
  } else {
    host.output_devices()?
        .find(|x| x.name().map(|y| y == opt.output_device).unwrap_or(false))
  }
      .expect("failed to find output device");

  println!("Using input device: \"{}\"", input_device.name()?);

  // We'll try and use the same configuration between streams to keep it simple.
  let config: cpal::StreamConfig = input_device.default_input_config()?.into();

  // Create a delay in case the input and output devices aren't synced.
  let latency_frames = (opt.latency / 1_000.0) * config.sample_rate.0 as f32;
  let latency_samples = latency_frames as usize * config.channels as usize;

  // The buffer to share samples
  let ring = RingBuffer::new(latency_samples * 2);
  let (mut producer, mut consumer) = ring.split();

  // Fill the samples with 0.0 equal to the length of the delay.
  for _ in 0..latency_samples {
    // The ring buffer has twice as much space as necessary to add latency here,
    // so this should never fail
    producer.push(0.0).unwrap();
  }

  let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
    let mut output_fell_behind = false;

    // print input volume
    let mut sum = 0.0;
    for i in 0..data.len() {
      sum += data[i].powi(2);
    }
    let volume = sum.sqrt() as i32 * 5;
    println!("input volume: {}", volume);

  };

  let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
    let mut input_fell_behind = false;
    for sample in data {
      *sample = match consumer.pop() {
        Some(s) => s,
        None => {
          input_fell_behind = true;
          0.0
        }
      };
    }
    if input_fell_behind {
      eprintln!("input stream fell behind: try increasing latency");
    }
  };

  // Build streams.
  println!(
    "Attempting to build both streams with f32 samples and `{:?}`.",
    config
  );
  let input_stream = input_device.build_input_stream(&config, input_data_fn, err_fn)?;
  println!("Successfully built streams.");

  // Play the streams.
  println!(
    "Starting the input and output streams with `{}` milliseconds of latency.",
    opt.latency
  );
  input_stream.play()?;

}

fn err_fn(_err: cpal::StreamError) {
  eprintln!("an error occurred on stream");
}