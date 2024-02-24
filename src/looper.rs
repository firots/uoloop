use std::sync::mpsc::Receiver;
use mouse_position::mouse_position::Mouse;
use winapi::um::winuser::{mouse_event, GetForegroundWindow, MapVirtualKeyExW, SendMessageW, SetCursorPos, MAPVK_VK_TO_VSC, WM_KEYDOWN};
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
            mouse_event(key_down.into(), x, y, 0, 0);
            std::thread::sleep(std::time::Duration::from_millis(5));
            mouse_event(key_up.into(), x, y, 0, 0);
            if double_click {
                std::thread::sleep(std::time::Duration::from_millis(5));
                mouse_event(key_down.into(), x, y, 0, 0);
                std::thread::sleep(std::time::Duration::from_millis(5));
                mouse_event(key_up.into(), x, y, 0, 0);
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
            if let Mouse::Position { x, y } = old_mouse_position {
                SetCursorPos(x, y);
            } else {
                println!("{}", CURSOR_POSITION_NOT_FOUND);
            }
        }
    }

    unsafe fn send_key_press(&self, key: u16) {
        let scan_code = 1 | ( MapVirtualKeyExW(key.into(), MAPVK_VK_TO_VSC, self.keyboard_handle as _) << 16);
        SendMessageW(self.window_handle as _, WM_KEYDOWN, key as _, scan_code as _);
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