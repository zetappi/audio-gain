mod audio;

use audio::{Sink, get_mute, get_volume, list_sinks, set_mute, set_volume};
use eframe::egui;
use std::time::{Duration, Instant};

struct AudioGainApp {
    sinks: Vec<Sink>,
    selected_sink: Option<String>,
    volume: f32,
    mute: bool,
    last_refresh: Instant,
    status: String,
}

impl Default for AudioGainApp {
    fn default() -> Self {
        let sinks = list_sinks();
        let selected = audio::default_sink().or_else(|| sinks.first().map(|s| s.name.clone()));
        let (volume, mute) = if let Some(s) = &selected {
            (get_volume(s).unwrap_or(100.0), get_mute(s))
        } else {
            (100.0, false)
        };
        Self {
            sinks,
            selected_sink: selected,
            volume,
            mute,
            last_refresh: Instant::now(),
            status: String::new(),
        }
    }
}

impl AudioGainApp {
    fn refresh(&mut self) {
        self.sinks = list_sinks();
        if self.selected_sink.is_none() {
            self.selected_sink = audio::default_sink().or_else(|| self.sinks.first().map(|s| s.name.clone()));
        }
        if let Some(s) = &self.selected_sink {
            if let Some(v) = get_volume(s) {
                self.volume = v;
            }
            self.mute = get_mute(s);
        }
    }
}

impl eframe::App for AudioGainApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if self.last_refresh.elapsed() > Duration::from_millis(750) {
            self.refresh();
            self.last_refresh = Instant::now();
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Audio Gain - Amplificatore Volume");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Sink:");
                let mut changed = false;
                egui::ComboBox::from_label("")
                    .selected_text(
                        self.selected_sink
                            .as_ref()
                            .and_then(|n| self.sinks.iter().find(|s| &s.name == n))
                            .map(|s| s.description.clone())
                            .unwrap_or_else(|| "nessuno".to_string()),
                    )
                    .show_ui(ui, |ui| {
                        for s in &self.sinks {
                            if ui
                                .selectable_value(&mut self.selected_sink, Some(s.name.clone()), &s.description)
                                .changed()
                            {
                                changed = true;
                            }
                        }
                    });
                if changed {
                    self.refresh();
                }
            });

            ui.add_space(8.0);
            ui.label(format!("Volume: {:.0}%", self.volume));
            let slider = egui::Slider::new(&mut self.volume, 0.0..=200.0)
                .suffix("%")
                .text("Gain");
            if ui.add(slider).changed()
                && let Some(s) = &self.selected_sink {
                    set_volume(s, self.volume);
                }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                for preset in [0.0_f32, 50.0, 100.0, 150.0, 200.0] {
                    if ui.button(format!("{:.0}%", preset)).clicked() {
                        self.volume = preset;
                        if let Some(s) = &self.selected_sink {
                            set_volume(s, self.volume);
                        }
                    }
                }
            });

            ui.add_space(8.0);
            if ui.checkbox(&mut self.mute, "Muto").changed()
                && let Some(s) = &self.selected_sink {
                    set_mute(s, self.mute);
                }

            ui.add_space(8.0);
            if ui.button("Aggiorna").clicked() {
                self.refresh();
            }
            if !self.status.is_empty() {
                ui.label(&self.status);
            }
        });

        ctx.request_repaint_after(Duration::from_millis(250));
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([420.0, 320.0])
            .with_title("Audio Gain"),
        ..Default::default()
    };
    eframe::run_native(
        "Audio Gain",
        options,
        Box::new(|_cc| Ok(Box::new(AudioGainApp::default()))),
    )
}