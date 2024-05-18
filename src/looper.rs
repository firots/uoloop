use std::sync::mpsc::Receiver;
use mouse_position::mouse_position::Mouse;
use winapi::um::winuser::{mouse_event, GetForegroundWindow, MapVirtualKeyExW, SendMessageW, SetCursorPos, MAPVK_VK_TO_VSC, WM_KEYDOWN, WM_KEYUP};
use crate::{constants::CURSOR_POSITION_NOT_FOUND, input_key::UserInput};

pub enum LooperMessage {
    Stop
}

pub struct LooperStep {
    pub(crate) user_input: UserInput,
    pub(crate) wait_time: u64,
    pub(crate) pos_x: u32,
    pub(crate) pos_y: u32,
}

pub struct Looper {
    pub(crate) window_handle: usize,
    pub(crate) steps: Vec<LooperStep>,
    pub(crate) receiver: Receiver<LooperMessage>,
    pub(crate) keyboard_handle: usize
}

impl Looper {
    pub fn start(&self) {
        'outer: loop {
            for step in &self.steps {
                if let Ok(message) = self.receiver.try_recv() {
                    match message {
                        LooperMessage::Stop => break 'outer
                    }
                }
                self.send_input(step);
                std::thread::sleep(std::time::Duration::from_millis(step.wait_time));
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    fn is_game_on_foreground(&self) -> bool {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                false
            } else {
                let hwnd = hwnd as usize;
                hwnd == self.window_handle
            }
        }
    }

    unsafe fn send_mouse_click(&self, x: u32, y: u32, key_down: u16, key_up: u16, double_click: bool) {
        if self.is_game_on_foreground() {
            let old_mouse_position = Mouse::get_mouse_position();
            SetCursorPos(x.try_into().unwrap(), y.try_into().unwrap());
            std::thread::sleep(std::time::Duration::from_millis(5));
            self.mouse_event_if_on_macro_position(key_down.into(), x, y);
            std::thread::sleep(std::time::Duration::from_millis(5));
            self.mouse_event_if_on_macro_position(key_up.into(), x, y);
            if double_click {
                std::thread::sleep(std::time::Duration::from_millis(5));
                self.mouse_event_if_on_macro_position(key_down.into(), x, y);
                std::thread::sleep(std::time::Duration::from_millis(5));
                self.mouse_event_if_on_macro_position(key_up.into(), x, y);
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
            if let Mouse::Position { x, y } = old_mouse_position {
                SetCursorPos(x, y);
            } else {
                println!("{}", CURSOR_POSITION_NOT_FOUND);
            }
        }
    }

    unsafe fn mouse_event_if_on_macro_position(&self, key: u16, x: u32, y: u32) {
        if self.is_cursor_on_macro_position(x, y) {
            mouse_event(key.into(), x, y, 0, 0);
        }
    }

    fn is_cursor_on_macro_position(&self, macro_x: u32, macro_y: u32) -> bool {
        let position = Mouse::get_mouse_position();
        let is_on_macro_position: bool;
        match position {
            Mouse::Position { x, y } => {
                is_on_macro_position = x == macro_x.try_into().unwrap() && y == macro_y.try_into().unwrap()
            }
            Mouse::Error => is_on_macro_position = false
        };
        is_on_macro_position
    }

    unsafe fn send_key_press(&self, key: u16) {
        let scan_code = 1 | ( MapVirtualKeyExW(key.into(), MAPVK_VK_TO_VSC, self.keyboard_handle as _) << 16);
        SendMessageW(self.window_handle as _, WM_KEYDOWN, key as _, scan_code as _);
        std::thread::sleep(std::time::Duration::from_millis(10));
        SendMessageW(self.window_handle as _, WM_KEYUP, key as _, scan_code as _);
    }

    fn send_input(&self, step: &LooperStep) {
        match step.user_input {
            UserInput::SingleClick { key_down, key_up, ..} => {
                unsafe {
                    self.send_mouse_click(step.pos_x.try_into().unwrap(), step.pos_y.try_into().unwrap(), key_down, key_up, false);
                } 
            }
            UserInput::DoubleClick { key_down, key_up, ..} => {
                unsafe {
                    self.send_mouse_click(step.pos_x.try_into().unwrap(), step.pos_y.try_into().unwrap(), key_down, key_up, true);
                }
            }
            UserInput::KeyPress { key, ..} => {
                unsafe {
                    self.send_key_press(key);
                }
            }
        }
    }
}