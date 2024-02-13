use std::env;
use std::{sync::mpsc::{self, Receiver, Sender}, thread};
use egui::Vec2;
use eframe::egui;
use winapi::um::winuser::{GetWindowTextLengthW, GetWindowTextW, MapVirtualKeyExW, SendMessageW, MAPVK_VK_TO_VSC, WM_KEYDOWN};

extern "system" {
    fn GetKeyboardLayout(thread_id: u32) -> usize;
    fn GetCurrentThreadId() -> u32;
}

fn main() -> Result<(), eframe::Error> {
    let args: Vec<String> = env::args().collect();
    let window_handle: usize = args[1].clone().parse().unwrap();
    let window_text = get_window_title(window_handle);
    let player_name = extract_name(&window_text);
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([350.0, 142.0])
            .with_resizable(false)
            .with_maximize_button(false),
        ..Default::default()
    };

    eframe::run_native(
        &("Valor Loop - ".to_owned() + player_name),
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

struct MainScreen {
    window_handle: usize,
    sender: Option<Sender<LooperMessage>>,
    looping: bool,
    loop_values: Vec<SelectedLoopValue>
}

#[derive(Clone)]
struct SelectedLoopValue {
    selected_key_text: String,
    wait_time_text: String
}

impl eframe::App for MainScreen {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().combo_width = 135.0;
                ui.spacing_mut().text_edit_width = 100.0;
                ui.spacing_mut().button_padding = Vec2::new(20.0, 0.0);

                let mut combo_id = 0;
                for loop_value in &mut self.loop_values {
                    ui.horizontal(|ui| {
                        ui.add_enabled_ui(!self.looping, |ui| {
                            ui.label(format!("{}", "Tuş:"));
                            egui::ComboBox::new("tus".to_owned() + &combo_id.to_string(), "")
                            .selected_text(loop_value.selected_key_text.clone())
                            .show_ui(ui, |ui| {
                                for input_key in &INPUT_KEYS {
                                    ui.selectable_value(&mut loop_value.selected_key_text, input_key.title.to_owned(), input_key.title);
                                }
                            });
                            ui.label(format!("{}", "Bekle:"));
                            ui.text_edit_singleline(&mut loop_value.wait_time_text);
                            ui.spacing();
                        });
                        combo_id += 1;
                    });
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.horizontal(|ui| {
                        let button_text: &str;
                        if self.looping {
                            button_text = "Dur";
                        } else {
                            button_text = "Başla";
                        }
                        ui.add_space(2.0);
                        if ui.button(button_text).clicked() {
                            self.on_button_tap();
                        }
                    });
                })


            });
        });
    }
}

impl MainScreen {
    fn new(window_handle: usize) -> Self {
        let selected_value = SelectedLoopValue { 
            selected_key_text: String::from(""), 
            wait_time_text: String::from("100") 
        };
        Self {
            window_handle,
            sender: None,
            looping: false,
            loop_values: vec![selected_value; 5]
        }
    }
}

impl MainScreen {
    fn on_button_tap(&mut self) {
        if self.looping {
            self.stop_loop()
        } else {
            self.start_loop()
        }
    }

    fn start_loop(&mut self) {
        let mut steps = Vec::new();
        for loop_value in &mut self.loop_values {
            let key_index = INPUT_KEYS
                .iter()
                .position(|r| r.title == loop_value.selected_key_text)
                .unwrap();
                
            if key_index > 0 {
                let key = INPUT_KEYS[key_index].key;
                let mut wait_time = loop_value.wait_time_text.parse().unwrap_or(100);
                if wait_time < 100 {
                    wait_time = 100;
                }
                loop_value.wait_time_text = wait_time.to_string();
                let looper_step = LooperStep {
                    key,
                    wait_time
                };
                steps.push(looper_step);
            } else {
                loop_value.wait_time_text = "0".to_owned();
            }
        }

        let (tx, rx) = mpsc::channel();
        self.sender = Some(tx.clone());
        let thread_id = unsafe { GetCurrentThreadId() };
        let keyboard_handle = unsafe { GetKeyboardLayout(thread_id) };

        let window_handle = self.window_handle.clone();
        let _ = thread::spawn(move || {
            let looper = Looper {
                window_handle,
                steps,
                receiver: rx,
                keyboard_handle
            };
            looper.start();
        });
        self.looping = true
    }

    fn stop_loop(&mut self) {
        if let Some(sender) = &self.sender {
            sender.send(LooperMessage::Stop).unwrap();
            self.looping = false
        }
    }
} 

struct LooperStep {
    key: u16,
    wait_time: u64,
}

struct Looper {
    window_handle: usize,
    steps: Vec<LooperStep>,
    receiver: Receiver<LooperMessage>,
    keyboard_handle: usize
}

impl Looper {
    fn start(&self) {
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

enum LooperMessage {
    Stop
}

struct InputKey {
    key: u16,
    title: &'static str,
}

static INPUT_KEYS: [InputKey; 74] = [
    InputKey { key: 0x0, title: ""},
    InputKey { key: 0x70, title: "F1" },
    InputKey { key: 0x71, title: "F2" },
    InputKey { key: 0x72, title: "F3" },
    InputKey { key: 0x73, title: "F4" },
    InputKey { key: 0x74, title: "F5" },
    InputKey { key: 0x75, title: "F6" },
    InputKey { key: 0x76, title: "F7" },
    InputKey { key: 0x77, title: "F8" },
    InputKey { key: 0x78, title: "F9" },
    InputKey { key: 0x79, title: "F10" },
    InputKey { key: 0x7A, title: "F11" },
    InputKey { key: 0x7B, title: "F12" },
    InputKey { key: 0x30, title: "0" },
    InputKey { key: 0x31, title: "1" },
    InputKey { key: 0x32, title: "2" },
    InputKey { key: 0x33, title: "3" },
    InputKey { key: 0x34, title: "4" },
    InputKey { key: 0x35, title: "5" },
    InputKey { key: 0x36, title: "6" },
    InputKey { key: 0x37, title: "7" },
    InputKey { key: 0x38, title: "8" },
    InputKey { key: 0x39, title: "9" },
    InputKey { key: 0x41, title: "A" },
    InputKey { key: 0x42, title: "B" },
    InputKey { key: 0x43, title: "C" },
    InputKey { key: 0x44, title: "D" },
    InputKey { key: 0x45, title: "E" },
    InputKey { key: 0x46, title: "F" },
    InputKey { key: 0x47, title: "G" },
    InputKey { key: 0x48, title: "H" },
    InputKey { key: 0x49, title: "I" },
    InputKey { key: 0x4A, title: "J" },
    InputKey { key: 0x4B, title: "K" },
    InputKey { key: 0x4C, title: "L" },
    InputKey { key: 0x4D, title: "M" },
    InputKey { key: 0x4E, title: "N" },
    InputKey { key: 0x4F, title: "O" },
    InputKey { key: 0x50, title: "P" },
    InputKey { key: 0x51, title: "Q" },
    InputKey { key: 0x52, title: "R" },
    InputKey { key: 0x53, title: "S" },
    InputKey { key: 0x54, title: "T" },
    InputKey { key: 0x55, title: "U" },
    InputKey { key: 0x56, title: "V" },
    InputKey { key: 0x57, title: "W" },
    InputKey { key: 0x58, title: "X" },
    InputKey { key: 0x59, title: "Y" },
    InputKey { key: 0x5A, title: "Z" },
    InputKey { key: 0x08, title: "Backspace" },
    InputKey { key: 0x09, title: "Tab" },
    InputKey { key: 0x0C, title: "Clear" },
    InputKey { key: 0x0D, title: "Enter" },
    InputKey { key: 0x10, title: "Shift" },
    InputKey { key: 0x11, title: "Ctrl" },
    InputKey { key: 0x12, title: "Alt" },
    InputKey { key: 0x13, title: "Pause" },
    InputKey { key: 0x14, title: "Caps Lock" },
    InputKey { key: 0x1B, title: "Esc" },
    InputKey { key: 0x20, title: "Space" },
    InputKey { key: 0x21, title: "Page Up" },
    InputKey { key: 0x22, title: "Page Down" },
    InputKey { key: 0x23, title: "End" },
    InputKey { key: 0x24, title: "Home" },
    InputKey { key: 0x25, title: "Left Arrow" },
    InputKey { key: 0x26, title: "Up Arrow" },
    InputKey { key: 0x27, title: "Right Arrow" },
    InputKey { key: 0x28, title: "Down Arrow" },
    InputKey { key: 0x29, title: "Select" },
    InputKey { key: 0x2A, title: "Print" },
    InputKey { key: 0x2B, title: "Execute" },
    InputKey { key: 0x2C, title: "Print Screen" },
    InputKey { key: 0x2D, title: "Insert" },
    InputKey { key: 0x2E, title: "Delete" },
];