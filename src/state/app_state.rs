use crate::analysis::FftAnalyzer;
use crate::audio::{AudioPlayer, PlaybackStatus};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

pub struct AppState {
    pub audio_player: Arc<AudioPlayer>,
    pub volume: Arc<Mutex<f32>>,
    fft_analyzer: Arc<Mutex<FftAnalyzer>>,
    spectrum_data: Arc<Mutex<Vec<f32>>>,
    analysis_thread_running: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        let audio_player = Arc::new(AudioPlayer::new()?);
        let volume = Arc::new(Mutex::new(0.5)); // Default volume at 50%
        let fft_analyzer = Arc::new(Mutex::new(FftAnalyzer::new()));
        let spectrum_data = Arc::new(Mutex::new(vec![0.0; 128]));
        let analysis_thread_running = Arc::new(AtomicBool::new(false));

        let state = Self {
            audio_player: Arc::clone(&audio_player),
            volume,
            fft_analyzer,
            spectrum_data: Arc::clone(&spectrum_data),
            analysis_thread_running: Arc::clone(&analysis_thread_running),
        };

        // Start the FFT analysis thread
        state.start_analysis_thread();

        Ok(state)
    }

    fn start_analysis_thread(&self) {
        let audio_player = Arc::clone(&self.audio_player);
        let fft_analyzer = Arc::clone(&self.fft_analyzer);
        let spectrum_data = Arc::clone(&self.spectrum_data);
        let running = Arc::clone(&self.analysis_thread_running);

        running.store(true, Ordering::Relaxed);

        thread::spawn(move || {
            // Target 60 Hz update rate
            let update_interval = Duration::from_millis(16);

            while running.load(Ordering::Relaxed) {
                let start = std::time::Instant::now();

                // Get audio samples from the player
                let analyzer = fft_analyzer.lock().unwrap();
                let required_samples = analyzer.required_samples();
                drop(analyzer);

                let (samples, channels) = audio_player.get_samples(required_samples);
                let sample_rate = audio_player.get_sample_rate();

                // Process FFT if we have enough samples
                if samples.len() >= required_samples {
                    let mut analyzer = fft_analyzer.lock().unwrap();
                    let spectrum = analyzer.process(&samples, channels, sample_rate);

                    // Update shared spectrum data
                    let mut data = spectrum_data.lock().unwrap();
                    data.clear();
                    data.extend_from_slice(spectrum);
                }

                // Sleep to maintain update rate
                let elapsed = start.elapsed();
                if elapsed < update_interval {
                    thread::sleep(update_interval - elapsed);
                }
            }
        });
    }

    pub fn load_file(&self, path: PathBuf) -> anyhow::Result<()> {
        self.audio_player.load_file(path)?;

        // Apply current volume to newly loaded file
        let volume = *self.volume.lock().unwrap();
        self.audio_player.set_volume(volume)?;

        Ok(())
    }

    pub fn play(&self) -> anyhow::Result<()> {
        self.audio_player.play()?;
        Ok(())
    }

    pub fn pause(&self) -> anyhow::Result<()> {
        self.audio_player.pause()?;
        Ok(())
    }

    pub fn stop(&self) -> anyhow::Result<()> {
        self.audio_player.stop()?;
        Ok(())
    }

    pub fn set_volume(&self, volume: f32) -> anyhow::Result<()> {
        let clamped_volume = volume.clamp(0.0, 1.0);
        *self.volume.lock().unwrap() = clamped_volume;
        self.audio_player.set_volume(clamped_volume)?;
        Ok(())
    }

    pub fn get_volume(&self) -> f32 {
        *self.volume.lock().unwrap()
    }

    pub fn get_status(&self) -> PlaybackStatus {
        self.audio_player.get_status()
    }

    pub fn get_current_file(&self) -> Option<PathBuf> {
        self.audio_player.get_current_file()
    }

    pub fn get_current_filename(&self) -> Option<String> {
        self.get_current_file()
            .and_then(|path| path.file_name().map(|s| s.to_string_lossy().to_string()))
    }

    pub fn is_finished(&self) -> bool {
        self.audio_player.is_finished()
    }

    /// Get the current spectrum data for visualization
    pub fn get_spectrum(&self) -> Vec<f32> {
        self.spectrum_data.lock().unwrap().clone()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new().expect("Failed to create default app state")
    }
}

impl Drop for AppState {
    fn drop(&mut self) {
        // Signal the analysis thread to stop
        self.analysis_thread_running.store(false, Ordering::Relaxed);
    }
}
