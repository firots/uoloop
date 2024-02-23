use std::{sync::mpsc::{self, Sender}, thread};
use egui::Vec2;
use mouse_position::mouse_position::Mouse;
use winapi::um::{processthreadsapi::GetCurrentThreadId, winuser::GetKeyboardLayout};
use crate::{constants::*, input_key::USER_INPUTS, looper::{Looper, LooperMessage, LooperStep}};

#[derive(Clone)]
struct SelectedLoopValue {
    selected_key_text: String,
    wait_time_text: String,
    pos_x_text: String,
    pos_y_text: String,
}

pub struct MainScreen {
    window_handle: usize,
    sender: Option<Sender<LooperMessage>>,
    looping: bool,
    loop_values: Vec<SelectedLoopValue>
}

impl MainScreen {
    pub fn new(window_handle: usize) -> Self {
        let selected_value = SelectedLoopValue { 
            selected_key_text: SELECTED_KEY_NONE_TEXT.to_string(),
            wait_time_text: DEFAULT_WAIT_TIME.to_string(),
            pos_x_text: DEFAULT_POS_X.to_string(),
            pos_y_text: DEFAULT_POS_Y.to_string()
        };
        Self {
            window_handle,
            sender: None,
            looping: false,
            loop_values: vec![selected_value; 5]
        }
    }
}

impl eframe::App for MainScreen {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().combo_width = 155.0;
                ui.spacing_mut().text_edit_width = 50.0;
                ui.spacing_mut().button_padding = Vec2::new(20.0, 0.0);
                self.loop_views(ui);
                self.start_button_view(ui);
                ctx.request_repaint();
            });
        });
    }
}

impl MainScreen {
    fn loop_views(&mut self, ui: &mut egui::Ui) {
        let mut combo_id = 0;
        for loop_value in &mut self.loop_values {
            ui.horizontal(|ui| {
                ui.add_enabled_ui(!self.looping, |ui| {
                    ui.label(format!("{}", SELECTED_KEY_TITLE));
                    egui::ComboBox::new("tus".to_owned() + &combo_id.to_string(), "")
                    .selected_text(loop_value.selected_key_text.clone())
                    .show_ui(ui, |ui| {
                        for input_key in &USER_INPUTS {
                            ui.selectable_value(&mut loop_value.selected_key_text, input_key.get_title().to_owned(), input_key.get_title().to_owned());
                        }
                    });
                    ui.spacing();

                    ui.label(format!("{}", X_LABEL_TITLE));
                    ui.text_edit_singleline(&mut loop_value.pos_x_text);
                    ui.spacing();

                    ui.label(format!("{}", Y_LABEL_TITLE));
                    ui.text_edit_singleline(&mut loop_value.pos_y_text);
                    ui.spacing();

                    ui.label(format!("{}", WAIT_TITLE));
                    ui.text_edit_singleline(&mut loop_value.wait_time_text);
                    ui.spacing();
                });
                combo_id += 1;
            });
        }
    }

    fn start_button_view(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            ui.horizontal(|ui| {
                let button_text = if self.looping {
                    STOP_BUTTON_TITLE
                } else {
                    START_BUTTON_TITLE
                };
                if ui.button(button_text).clicked() {
                    self.on_button_tap();
                }

                let position = Mouse::get_mouse_position();
                match position {
                    Mouse::Position { x, y } => ui.label(format!("X: {}, Y: {}", x, y)),
                    Mouse::Error => ui.label(CURSOR_POSITION_NOT_FOUND)
               }
            });
        });
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

    fn get_loop_steps(&mut self) -> Vec<LooperStep>{
        let mut steps = Vec::new();
        for loop_value in &mut self.loop_values {
            let key_index = USER_INPUTS
                .iter()
                .position(|r| r.get_title() == loop_value.selected_key_text)
                .unwrap();
                
            if key_index > 0 {
                let user_input = &USER_INPUTS[key_index];
                let mut wait_time = loop_value.wait_time_text.parse().unwrap_or(100);
                let pos_x = loop_value.pos_x_text.parse().unwrap_or(0);
                let pos_y = loop_value.pos_y_text.parse().unwrap_or(0);
                if wait_time < MINIMUM_WAIT_TIME {
                    wait_time = MINIMUM_WAIT_TIME;
                }
                loop_value.wait_time_text = wait_time.to_string();
                let looper_step = LooperStep {
                    user_input: user_input.clone(),
                    wait_time,
                    pos_x,
                    pos_y
                };
                steps.push(looper_step);
            } else {
                loop_value.wait_time_text = WAIT_TIME_NONE_TEXT.to_owned();
            }
        }
        steps
    }

    fn start_loop(&mut self) {
        let steps = self.get_loop_steps();
        if steps.is_empty() { return }
        let (tx, rx) = mpsc::channel();
        self.sender = Some(tx.clone());
        let thread_id = unsafe { GetCurrentThreadId() };
        let keyboard_handle = unsafe { GetKeyboardLayout(thread_id) } as usize;
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