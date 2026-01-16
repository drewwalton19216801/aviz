use crate::audio::PlaybackStatus;
use crate::state::AppState;
use eframe::egui;
use std::sync::Arc;

pub struct MainWindow {
    state: Arc<AppState>,
}

impl MainWindow {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Configure egui style
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.button_padding = egui::vec2(8.0, 4.0);
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        cc.egui_ctx.set_style(style);

        Self {
            state: Arc::new(AppState::new().expect("Failed to create app state")),
        }
    }
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check if playback finished and update status
        if self.state.get_status() == PlaybackStatus::Playing && self.state.is_finished() {
            let _ = self.state.stop();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Audio Visualizer");
            ui.add_space(10.0);

            // File loading section
            ui.group(|ui| {
                ui.label("Audio File:");
                ui.horizontal(|ui| {
                    if ui.button("Load File").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Audio Files", &["mp3", "wav"])
                            .pick_file()
                        {
                            match self.state.load_file(path) {
                                Ok(_) => {},
                                Err(e) => {
                                    tracing::error!("Failed to load file: {}", e);
                                }
                            }
                        }
                    }

                    if let Some(filename) = self.state.get_current_filename() {
                        ui.label(filename);
                    } else {
                        ui.label("No file loaded");
                    }
                });
            });

            ui.add_space(10.0);

            // Playback controls section
            ui.group(|ui| {
                ui.label("Playback Controls:");
                ui.horizontal(|ui| {
                    let status = self.state.get_status();
                    let has_file = self.state.get_current_file().is_some();

                    // Play button
                    if ui
                        .add_enabled(
                            has_file && status != PlaybackStatus::Playing,
                            egui::Button::new("▶ Play"),
                        )
                        .clicked()
                    {
                        if let Err(e) = self.state.play() {
                            tracing::error!("Failed to play: {}", e);
                        }
                    }

                    // Pause button
                    if ui
                        .add_enabled(
                            has_file && status == PlaybackStatus::Playing,
                            egui::Button::new("⏸ Pause"),
                        )
                        .clicked()
                    {
                        if let Err(e) = self.state.pause() {
                            tracing::error!("Failed to pause: {}", e);
                        }
                    }

                    // Stop button
                    if ui
                        .add_enabled(
                            has_file && status != PlaybackStatus::Stopped,
                            egui::Button::new("⏹ Stop"),
                        )
                        .clicked()
                    {
                        if let Err(e) = self.state.stop() {
                            tracing::error!("Failed to stop: {}", e);
                        }
                    }
                });
            });

            ui.add_space(10.0);

            // Volume control section
            ui.group(|ui| {
                ui.label("Volume:");
                let mut volume = self.state.get_volume();
                if ui
                    .add(egui::Slider::new(&mut volume, 0.0..=1.0).text(""))
                    .changed()
                {
                    if let Err(e) = self.state.set_volume(volume) {
                        tracing::error!("Failed to set volume: {}", e);
                    }
                }
                ui.label(format!("{}%", (volume * 100.0) as i32));
            });

            ui.add_space(10.0);

            // Status display section
            ui.group(|ui| {
                ui.label("Status:");
                let status_text = match self.state.get_status() {
                    PlaybackStatus::Playing => "Playing",
                    PlaybackStatus::Paused => "Paused",
                    PlaybackStatus::Stopped => "Stopped",
                };
                ui.label(status_text);
            });
        });

        // Request repaint for smooth UI updates
        ctx.request_repaint();
    }
}
