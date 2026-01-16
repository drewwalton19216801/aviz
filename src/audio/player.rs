use crate::utils::{AudioError, Result};
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackStatus {
    Stopped,
    Playing,
    Paused,
}

pub struct AudioPlayer {
    stream: Arc<OutputStream>,
    sink: Arc<Mutex<Option<Sink>>>,
    current_file: Arc<Mutex<Option<PathBuf>>>,
    status: Arc<Mutex<PlaybackStatus>>,
}

impl AudioPlayer {
    pub fn new() -> Result<Self> {
        let stream = OutputStreamBuilder::open_default_stream()
            .map_err(|e| AudioError::PlaybackError(format!("Failed to create audio stream: {}", e)))?;

        Ok(Self {
            stream: Arc::new(stream),
            sink: Arc::new(Mutex::new(None)),
            current_file: Arc::new(Mutex::new(None)),
            status: Arc::new(Mutex::new(PlaybackStatus::Stopped)),
        })
    }

    pub fn load_file(&self, path: PathBuf) -> Result<()> {
        // Open the file
        let file = File::open(&path)
            .map_err(|e| AudioError::LoadError(format!("Failed to open file: {}", e)))?;
        
        let buf_reader = BufReader::new(file);
        
        // Decode the audio file
        let source = Decoder::new(buf_reader)
            .map_err(|e| AudioError::DecodeError(format!("Failed to decode audio: {}", e)))?;

        // Clear the current sink and create a new one
        let mut sink_guard = self.sink.lock().unwrap();
        if let Some(sink) = sink_guard.take() {
            sink.stop();
        }

        // Create a new sink connected to the mixer
        let new_sink = Sink::connect_new(self.stream.mixer());
        
        // Append the source to the sink
        new_sink.append(source);
        
        // Pause immediately so it doesn't start playing
        new_sink.pause();

        // Store the sink and file path
        *sink_guard = Some(new_sink);
        drop(sink_guard);
        
        *self.current_file.lock().unwrap() = Some(path);
        *self.status.lock().unwrap() = PlaybackStatus::Stopped;

        Ok(())
    }

    pub fn play(&self) -> Result<()> {
        let sink_guard = self.sink.lock().unwrap();
        
        if let Some(sink) = sink_guard.as_ref() {
            sink.play();
            drop(sink_guard);
            *self.status.lock().unwrap() = PlaybackStatus::Playing;
            Ok(())
        } else {
            Err(AudioError::NoFileLoaded)
        }
    }

    pub fn pause(&self) -> Result<()> {
        let sink_guard = self.sink.lock().unwrap();
        
        if let Some(sink) = sink_guard.as_ref() {
            sink.pause();
            drop(sink_guard);
            *self.status.lock().unwrap() = PlaybackStatus::Paused;
            Ok(())
        } else {
            Err(AudioError::NoFileLoaded)
        }
    }

    pub fn stop(&self) -> Result<()> {
        let sink_guard = self.sink.lock().unwrap();
        
        if let Some(sink) = sink_guard.as_ref() {
            sink.stop();
            drop(sink_guard);
            *self.status.lock().unwrap() = PlaybackStatus::Stopped;
            
            // Reload the file if we have one
            let current_file_guard = self.current_file.lock().unwrap();
            let current_file = current_file_guard.clone();
            drop(current_file_guard);
            
            if let Some(path) = current_file {
                self.load_file(path)?;
            }
            
            Ok(())
        } else {
            Err(AudioError::NoFileLoaded)
        }
    }

    pub fn set_volume(&self, volume: f32) -> Result<()> {
        let sink_guard = self.sink.lock().unwrap();
        
        if let Some(sink) = sink_guard.as_ref() {
            sink.set_volume(volume.clamp(0.0, 1.0));
            Ok(())
        } else {
            Err(AudioError::NoFileLoaded)
        }
    }

    pub fn get_status(&self) -> PlaybackStatus {
        *self.status.lock().unwrap()
    }

    pub fn get_current_file(&self) -> Option<PathBuf> {
        self.current_file.lock().unwrap().clone()
    }

    pub fn is_finished(&self) -> bool {
        let sink_guard = self.sink.lock().unwrap();
        
        if let Some(sink) = sink_guard.as_ref() {
            sink.empty()
        } else {
            true
        }
    }

    pub fn get_position(&self) -> Duration {
        // Note: rodio's Sink doesn't provide position tracking out of the box
        // This is a placeholder that returns zero
        // In Phase 2+, we could implement this by tracking samples
        Duration::from_secs(0)
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new().expect("Failed to create default audio player")
    }
}
