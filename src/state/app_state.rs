use crate::audio::{AudioPlayer, PlaybackStatus};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub audio_player: Arc<AudioPlayer>,
    pub volume: Arc<Mutex<f32>>,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        let audio_player = Arc::new(AudioPlayer::new()?);
        let volume = Arc::new(Mutex::new(0.5)); // Default volume at 50%

        Ok(Self {
            audio_player,
            volume,
        })
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
}

impl Default for AppState {
    fn default() -> Self {
        Self::new().expect("Failed to create default app state")
    }
}
