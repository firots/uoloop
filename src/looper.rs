use std::sync::mpsc::Receiver;
use winapi::um::winuser::{MapVirtualKeyExW, SendMessageW, MAPVK_VK_TO_VSC, WM_KEYDOWN};

pub enum LooperMessage {
    Stop
}

pub struct LooperStep {
    pub(crate) key: u16,
    pub(crate) wait_time: u64,
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
                self.send_key_down(step.key);
                std::thread::sleep(std::time::Duration::from_millis(step.wait_time));
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    fn send_key_down(&self, key: u16) {
        let scan_code = 1 | (unsafe { MapVirtualKeyExW(key.into(), MAPVK_VK_TO_VSC, self.keyboard_handle as _) } << 16);
        unsafe {
            SendMessageW(self.window_handle as _, WM_KEYDOWN, key as _, scan_code as _);
        }
    }
}