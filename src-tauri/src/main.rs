#![warn(clippy::all)]
#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

use std::ops::{Deref, DerefMut};

use enigo::*;
use parking_lot::Mutex;

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

#[tauri::command]
fn mouse_click(mouse: tauri::State<Mutex<Mouse>>) {
  mouse.lock().mouse_click(MouseButton::Left)
}

fn main() {
  tauri::Builder::default()
      .manage(Mutex::new(Mouse(Enigo::new())))
      .invoke_handler(tauri::generate_handler![mouse_click])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
}