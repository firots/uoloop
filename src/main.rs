#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use std::env;
use eframe::egui;
use uoloop::main_screen::MainScreen;
use winapi::um::winuser::{GetWindowTextLengthW, GetWindowTextW};
#[macro_use]
extern crate litcrypt;
use_litcrypt!("/q^+^'+^&%+%&%+&%++^'+^'&/%/&%/&%");

fn main() -> Result<(), eframe::Error> {
    let args: Vec<String> = env::args().collect();
    let window_handle: usize = args[1].clone().parse().unwrap();
    let window_text = get_window_title(window_handle);
    let player_name = extract_name(&window_text);
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([485.0, 142.0])
            .with_resizable(false)
            .with_maximize_button(false),
        ..Default::default()
    };

    eframe::run_native(
        &(lc!("Valor Loop - ").to_owned() + player_name),
        options,
        Box::new(move |_| Box::new(MainScreen::new(window_handle))),
    )
}

fn get_window_title(hwnd: usize) -> String {
    let length = unsafe { GetWindowTextLengthW(hwnd as _) };
    if length == 0 {
        return String::new();
    }

    let mut buffer: Vec<u16> = vec![0; (length + 1) as usize];
    unsafe {
        GetWindowTextW(hwnd as _, buffer.as_mut_ptr(), (length + 1) as i32);
    }
    String::from_utf16_lossy(&buffer)
}

fn extract_name(input: &str) -> &str {
    let parts: Vec<&str> = input.split('-').collect();
    if let Some(name) = parts.get(0) {
        return name.trim();
    }
    ""
}