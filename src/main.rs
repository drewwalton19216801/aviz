//! Audio Visualizer (aviz) - Real-time audio spectrum visualization application.
//!
//! This application provides an interactive GUI for playing audio files (MP3/WAV)
//! and displaying their frequency spectrum in real-time using FFT analysis.
//!
//! # Features
//! - Audio playback with play/pause/stop controls
//! - Real-time FFT-based spectrum visualization
//! - Volume control
//! - Keyboard shortcuts (Space for play/pause)
//! - Color-coded frequency bars
//!
//! # Architecture
//! - `audio`: Audio decoding and playback engine
//! - `analysis`: FFT processing and spectrum analysis
//! - `gui`: User interface and visualization
//! - `state`: Application state management
//! - `utils`: Error handling and utilities

mod analysis;
mod audio;
mod gui;
mod state;
mod utils;

use gui::MainWindow;

fn main() -> anyhow::Result<()> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Configure the native window options
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 750.0])
            .with_min_inner_size([700.0, 600.0])
            .with_title("Audio Visualizer"),
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "Audio Visualizer",
        native_options,
        Box::new(|cc| Ok(Box::new(MainWindow::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run application: {}", e))?;

    Ok(())
}
