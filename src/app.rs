use std::time::Instant;
use chrono::*;

/// How often we repaint the demo app by default
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RunMode {
    /// This is the default for the demo.
    ///
    /// If this is selected, Egui is only updated if are input events
    /// (like mouse movements) or there are some animations in the GUI.
    ///
    /// Reactive mode saves CPU.
    ///
    /// The downside is that the UI can become out-of-date if something it is supposed to monitor changes.
    /// For instance, a GUI for a thermostat need to repaint each time the temperature changes.
    /// To ensure the UI is up to date you need to call `egui::Context::request_repaint()` each
    /// time such an event happens. You can also chose to call `request_repaint()` once every second
    /// or after every single frame - this is called `Continuous` mode,
    /// and for games and interactive tools that need repainting every frame anyway, this should be the default.
    Reactive,

    /// This will call `egui::Context::request_repaint()` at the end of each frame
    /// to request the backend to repaint as soon as possible.
    ///
    /// On most platforms this will mean that Egui will run at the display refresh rate of e.g. 60 Hz.
    ///
    /// For this demo it is not any reason to do so except to
    /// demonstrate how quickly Egui runs.
    ///
    /// For games or other interactive apps, this is probably what you want to do.
    /// It will guarantee that Egui is always up-to-date.
    Continuous,
}

/// Default for demo is Reactive since
/// 1) We want to use minimal CPU
/// 2) There are no external events that could invalidate the UI
///    so there are no events to miss.
impl Default for RunMode {
    fn default() -> Self {
        RunMode::Reactive
    }
}

pub struct Timer {
    is_running: bool,
    previous_timestamp: Instant,
    start_time: u128,
    timer: u128,
}

impl Timer {
    fn new(hours : u32, minutes: u32, seconds : u32) -> Timer
    {
        let seconds_in_ms = ((hours as u128 * 60 * 60) + (minutes as u128 * 60) + seconds as u128) * 1000;
        Timer {
            is_running: false,
            previous_timestamp: Instant::now(),
            start_time: seconds_in_ms,
            timer: seconds_in_ms,
        }
    }

    fn update(&mut self)
    {
        if !self.is_running {
            return;
        }

        let ms_difference = self.previous_timestamp.elapsed().as_millis(); 
        self.timer -= ms_difference;
        self.previous_timestamp = Instant::now();
    }

    fn start_timer(&mut self){
        self.previous_timestamp = Instant::now();
        self.is_running = true;
    }

    fn pause_timer(&mut self) {
        self.is_running = false;
    }

    fn stop_timer(&mut self) {
        self.is_running = false;
        self.timer = self.start_time;
    }

    fn get_hours_remaining(&self) -> u32 {
        let hours = (self.timer / 1000 / 60 / 60) % 60;
        hours as u32
    }

    fn get_minutes_remaining(&self) -> u32 {
        let minutes = (self.timer / 1000 / 60) % 60;
        minutes as u32
    }

    fn get_seconds_remaining(&self) -> u32 {
        let seconds = (self.timer / 1000) % 60;
        seconds as u32
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // this how you opt-out of serialization of a member
    #[serde(skip)]
    value: f32,

    #[serde(skip)]
    run_mode: RunMode,

    #[serde(skip)]
    active_timer_index: usize,

    #[serde(skip)]
    timers : Vec<Timer>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let pomodoro_timer = Timer::new(2, 23, 17);
        let pause_timer = Timer::new(0, 15, 30);
        Self {
            value: 2.7,
            run_mode: RunMode::Continuous,
            active_timer_index: 0,
            timers: vec![pomodoro_timer, pause_timer]
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let value = &mut self.value;
        let run_mode = self.run_mode;
        let active_timer_index = &mut self.active_timer_index;
        let timers = &mut self.timers;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                *value += 1.0;
            }

            let mut timer_switched = false;
            ui.horizontal(|ui|{
                if ui.button("Pomodoro").clicked() {
                    *active_timer_index = 0;
                    timer_switched = true;
                }
                if ui.button("Pause").clicked() {
                    *active_timer_index = 1;
                    timer_switched = true;
                }
            });

            let active_timer = &mut timers[*active_timer_index];

            if timer_switched {
                active_timer.stop_timer();
            }

            //ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.label(format!("{}:{}:{}", active_timer.get_hours_remaining(), active_timer.get_minutes_remaining(), active_timer.get_seconds_remaining()));
            //});

            ui.horizontal(|ui| {
                if active_timer.is_running {
                    if ui.button("pause timer").clicked() {
                        active_timer.pause_timer();
                    }
                }else {
                    if ui.button("start timer").clicked() {
                        active_timer.start_timer();
                    }
                }
            
                if ui.button("stop timer").clicked() {
                    active_timer.stop_timer();
                }
            });

            active_timer.update();

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to(
                        "eframe",
                        "https://github.com/emilk/egui/tree/master/crates/eframe",
                    );
                    ui.label(".");
                    egui::warn_if_debug_build(ui);
                });
            });
        });

        if run_mode == RunMode::Continuous {
            // Tell the backend to repaint as soon as possible
            ctx.request_repaint();
        }
    }

    
}