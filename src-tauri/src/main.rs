#![warn(clippy::all)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{
    ops::{Deref, DerefMut},
    sync::{
        mpsc::{channel, Sender},
        Arc,
    },
    thread,
};

use enigo::*;
use parking_lot::Mutex;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::RingBuffer;
use serde::Serialize;
use tauri::Manager;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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

struct State {
    mouse: Mouse,
    enabled: bool,
    threshold: i32,
    met: bool,
    one_click_setting: bool,
    one_click_state: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            mouse: Mouse(Enigo::new()),
            enabled: false,
            threshold: 30,
            met: false,
            one_click_setting: false,
            one_click_state: false,
        }
    }

    pub fn click(&mut self) {
        self.mouse.mouse_click(MouseButton::Left);
    }

    pub fn mouse_down(&mut self) {
        self.mouse.mouse_down(MouseButton::Left);
    }
    pub fn mouse_up(&mut self) {
        self.mouse.mouse_up(MouseButton::Left);
    }
}

#[derive(Serialize, Clone, Copy)]
struct Payload {
    volume: i32,
    met: bool,
}

#[tauri::command]
fn set_enabled(state: tauri::State<Arc<Mutex<State>>>, enable: bool) {
    state.lock().enabled = enable;
    println!("Enabled: {}", enable);
}

#[tauri::command]
fn set_threshold(state: tauri::State<Arc<Mutex<State>>>, threshold: f32) {
    state.lock().threshold = threshold as i32;
    println!("Threshold: {}", threshold);
}

#[tauri::command]
fn set_one_click(state: tauri::State<Arc<Mutex<State>>>, one_click: bool) {
    state.lock().one_click_setting = one_click;
    println!("One Click: {}", one_click);
}

fn main() -> Result<()> {
    let host = cpal::default_host();

    let input_device = host
        .default_input_device()
        .expect("failed to find input device");

    println!("Using input device: \"{}\"", input_device.name()?);

    // We'll try and use the same configuration between streams to keep it simple.
    let config: cpal::StreamConfig = input_device.default_input_config()?.into();

    // Create a delay in case the input and output devices aren't synced.
    let latency_frames = config.sample_rate.0 as f32;
    let latency_samples = latency_frames as usize * config.channels as usize;

    // The buffer to share samples
    let ring = RingBuffer::new(latency_samples * 2);
    let (mut producer, _) = ring.split();

    // Fill the samples with 0.0 equal to the length of the delay.
    for _ in 0..latency_samples {
        // The ring buffer has twice as much space as necessary to add latency here,
        // so this should never fail
        producer.push(0.0).unwrap();
    }

    let state = Arc::new(Mutex::new(State::new()));
    let (mut tx, rx) = channel::<Payload>();
    let state1 = Arc::clone(&state);

    let input_stream = input_device.build_input_stream(
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| input_fn(data, &mut tx, &state1),
        err_fn,
    )?;

    input_stream.play()?;

    tauri::Builder::default()
        .manage(state)
        .setup(|app| {
            let handle = app.app_handle();
            thread::spawn(move || loop {
                if let Ok(payload) = rx.recv() {
                    handle
                        .emit_all("threshold", payload)
                        .expect("Could not emit event")
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![set_enabled, set_threshold, set_one_click])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}

fn input_fn(data: &[f32], channel: &mut Sender<Payload>, state: &Mutex<State>) {
    let mut sum = 0.0;
    for i in 0..data.len() {
        sum += data[i].powi(2);
    }
    let volume = sum.sqrt() as i32 * 15;

    let mut state = state.lock();

    let met = volume >= state.threshold;

    if state.enabled {
        if met && state.one_click_setting && !state.one_click_state {
            state.mouse_down();
            state.one_click_state = true;
        } else if !met && state.one_click_setting && state.one_click_state {
            state.mouse_up();
            state.one_click_state = false;
        } else if met && !state.one_click_setting {
            state.click();
        }
    }

    if let Err(e) = channel.send(Payload { volume, met }) {
        eprintln!("errored at sending data: {}", e);
    }

}

fn err_fn(_err: cpal::StreamError) {
    eprintln!("an error occurred on stream");
}
