use std::sync::mpsc::Receiver;
use winapi::um::winuser::{MapVirtualKeyExW, SendMessageW, MAPVK_VK_TO_VSC, WM_KEYDOWN};
use crate::input_key::UserInput;

pub enum LooperMessage {
    Stop
}

pub struct LooperStep {
    pub(crate) user_input: UserInput,
    pub(crate) wait_time: u64,
    pub(crate) pos_x: u64,
    pub(crate) pos_y: u64,
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

    fn send_input(&self, step: &LooperStep) {
        match step.user_input {
            UserInput::SingleClick { key_down, key_up, ..} => {
                println!("single click x:{}:y{} down:{} up:{}", step.pos_x, step.pos_y, key_down, key_up);
                unsafe {
                    SendMessageW(self.window_handle as _, key_down.into(), 0, ((step.pos_y << 16) | step.pos_x & 0xFFFF).try_into().unwrap());
                    SendMessageW(self.window_handle as _, key_up.into(), 0, ((step.pos_y << 16) | step.pos_x & 0xFFFF).try_into().unwrap());
                }
            }
            UserInput::DoubleClick { key, ..} => {
                unsafe {
                    SendMessageW(self.window_handle as _, key.into(), 0, ((step.pos_y << 16) | step.pos_x).try_into().unwrap());
                }
            }
            UserInput::KeyPress { key, ..} => {
                let scan_code = 1 | (unsafe { MapVirtualKeyExW(key.into(), MAPVK_VK_TO_VSC, self.keyboard_handle as _) } << 16);
                unsafe {
                    SendMessageW(self.window_handle as _, WM_KEYDOWN, key as _, scan_code as _);
                }
            }
        }
    }
}