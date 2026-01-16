use crate::utils::{AudioError, Result};
use ringbuf::{traits::*, HeapRb};
use rodio::{cpal::Sample, Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
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

// Sample buffer size - enough for FFT analysis
const SAMPLE_BUFFER_SIZE: usize = 8192;

/// A wrapper source that captures audio samples into a ring buffer
/// Converts samples to f32 for analysis
struct CapturingSource<S> {
    source: S,
    sample_buffer: Arc<Mutex<HeapRb<f32>>>,
}

impl<S> CapturingSource<S> {
    fn new(source: S, sample_buffer: Arc<Mutex<HeapRb<f32>>>) -> Self {
        Self {
            source,
            sample_buffer,
        }
    }
}

impl<S> Iterator for CapturingSource<S>
where
    S: Source,
    S::Item: Sample,
{
    type Item = S::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sample) = self.source.next() {
            // Convert sample to f32 and push to ring buffer
            if let Ok(mut buffer) = self.sample_buffer.lock() {
                let f32_sample = sample.to_sample::<f32>();
                let _ = buffer.try_push(f32_sample);
            }
            Some(sample)
        } else {
            None
        }
    }
}

impl<S> Source for CapturingSource<S>
where
    S: Source,
    S::Item: Sample,
{
    fn current_span_len(&self) -> Option<usize> {
        self.source.current_span_len()
    }

    fn channels(&self) -> u16 {
        self.source.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.source.total_duration()
    }
}

pub struct AudioPlayer {
    stream: Arc<OutputStream>,
    sink: Arc<Mutex<Option<Sink>>>,
    current_file: Arc<Mutex<Option<PathBuf>>>,
    status: Arc<Mutex<PlaybackStatus>>,
    sample_buffer: Arc<Mutex<HeapRb<f32>>>,
    sample_rate: Arc<Mutex<u32>>,
    channels: Arc<Mutex<u16>>,
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
            sample_buffer: Arc::new(Mutex::new(HeapRb::new(SAMPLE_BUFFER_SIZE))),
            sample_rate: Arc::new(Mutex::new(44100)),
            channels: Arc::new(Mutex::new(2)),
        })
    }

    pub fn load_file(&self, path: PathBuf) -> Result<()> {
        // Open the file
        let file = File::open(&path)
            .map_err(|e| AudioError::LoadError(format!("Failed to open file: {}", e)))?;
        
        let buf_reader = BufReader::new(file);
        
        // Decode the audio file
        let decoder = Decoder::new(buf_reader)
            .map_err(|e| AudioError::DecodeError(format!("Failed to decode audio: {}", e)))?;

        // Store sample rate and channels
        *self.sample_rate.lock().unwrap() = decoder.sample_rate();
        *self.channels.lock().unwrap() = decoder.channels();

        // Wrap the source to capture samples
        let capturing_source = CapturingSource::new(decoder, Arc::clone(&self.sample_buffer));

        // Clear the current sink and create a new one
        let mut sink_guard = self.sink.lock().unwrap();
        if let Some(sink) = sink_guard.take() {
            sink.stop();
        }

        // Create a new sink connected to the mixer
        let new_sink = Sink::connect_new(self.stream.mixer());
        
        // Append the capturing source to the sink
        new_sink.append(capturing_source);
        
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

    /// Get a copy of recent audio samples for FFT analysis
    /// Returns samples and the number of channels
    pub fn get_samples(&self, count: usize) -> (Vec<f32>, u16) {
        let buffer = self.sample_buffer.lock().unwrap();
        let channels = *self.channels.lock().unwrap();
        
        let available = buffer.occupied_len();
        let to_read = count.min(available);
        
        let mut samples = Vec::with_capacity(to_read);
        
        // Read samples by popping and re-pushing (since ringbuf doesn't have random access)
        // We'll just collect what we can from the buffer
        let buffer_vec: Vec<f32> = buffer.iter().copied().collect();
        
        // Get the most recent samples
        if buffer_vec.len() >= to_read {
            samples.extend_from_slice(&buffer_vec[buffer_vec.len() - to_read..]);
        } else {
            samples.extend_from_slice(&buffer_vec);
        }
        
        (samples, channels)
    }

    pub fn get_sample_rate(&self) -> u32 {
        *self.sample_rate.lock().unwrap()
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new().expect("Failed to create default audio player")
    }
}
