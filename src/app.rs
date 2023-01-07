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
    // Example stuff:
    label: String,

    // this how you opt-out of serialization of a member
    #[serde(skip)]
    value: f32,

    #[serde(skip)]
    run_mode: RunMode,

    #[serde(skip)]
    pomodoro_timer : Timer
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            run_mode: RunMode::Continuous,
            pomodoro_timer: Timer::new(2, 23, 17),
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
        let label = &mut self.label;
        let value = &mut self.value;
        let run_mode = self.run_mode;
        let pomodoro_timer = &mut self.pomodoro_timer;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(label);
            });

            ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                *value += 1.0;
            }

            let local_time : DateTime<Local> = Local::now();
            let time_in_hours = local_time.hour() as i32;
            let time_in_minutes = local_time.minute() as i32;
            let time_in_seconds = local_time.second() as i32; 
            ui.label(format!("{}:{}:{}", time_in_hours, time_in_minutes, time_in_seconds));

            if ui.button("start timer").clicked() {
                pomodoro_timer.start_timer();
            }
            if ui.button("pause timer").clicked() {
                pomodoro_timer.pause_timer();
            }
            if ui.button("stop timer").clicked() {
                pomodoro_timer.stop_timer();
            }
            pomodoro_timer.update();

            ui.label(format!("pomodoro timer {}:{}:{}", pomodoro_timer.get_hours_remaining(), pomodoro_timer.get_minutes_remaining(), pomodoro_timer.get_seconds_remaining()));

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
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("eframe template");
            ui.hyperlink("https://github.com/emilk/eframe_template");
            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));
            egui::warn_if_debug_build(ui);
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }

        if run_mode == RunMode::Continuous {
            // Tell the backend to repaint as soon as possible
            ctx.request_repaint();
        }
    }

    
}