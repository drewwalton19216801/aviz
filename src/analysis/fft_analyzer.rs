use realfft::{RealFftPlanner, RealToComplex};
use std::sync::Arc;

const FFT_SIZE: usize = 4096;
const NUM_BARS: usize = 128;
const SMOOTHING_FACTOR: f32 = 0.7; // Higher = more smoothing

pub struct FftAnalyzer {
    fft: Arc<dyn RealToComplex<f32>>,
    input_buffer: Vec<f32>,
    output_buffer: Vec<rustfft::num_complex::Complex<f32>>,
    window: Vec<f32>,
    spectrum: Vec<f32>,
    smoothed_spectrum: Vec<f32>,
}

impl FftAnalyzer {
    pub fn new() -> Self {
        let mut planner = RealFftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);
        
        let input_buffer = vec![0.0; FFT_SIZE];
        let output_buffer = vec![rustfft::num_complex::Complex::new(0.0, 0.0); FFT_SIZE / 2 + 1];
        
        // Create Hann window to reduce spectral leakage
        let window = Self::create_hann_window(FFT_SIZE);
        
        let spectrum = vec![0.0; NUM_BARS];
        let smoothed_spectrum = vec![0.0; NUM_BARS];
        
        Self {
            fft,
            input_buffer,
            output_buffer,
            window,
            spectrum,
            smoothed_spectrum,
        }
    }
    
    /// Create a Hann window function
    fn create_hann_window(size: usize) -> Vec<f32> {
        (0..size)
            .map(|i| {
                let factor = std::f32::consts::PI * i as f32 / (size - 1) as f32;
                0.5 * (1.0 - factor.cos())
            })
            .collect()
    }
    
    /// Convert stereo samples to mono by averaging channels
    fn stereo_to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
        if channels == 1 {
            return samples.to_vec();
        }
        
        samples
            .chunks(channels as usize)
            .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
            .collect()
    }
    
    /// Process audio samples and return frequency spectrum
    pub fn process(&mut self, samples: &[f32], channels: u16, sample_rate: u32) -> &[f32] {
        // Convert to mono if needed
        let mono_samples = Self::stereo_to_mono(samples, channels);
        
        // Need at least FFT_SIZE samples
        if mono_samples.len() < FFT_SIZE {
            return &self.smoothed_spectrum;
        }
        
        // Take the most recent FFT_SIZE samples
        let start_idx = mono_samples.len() - FFT_SIZE;
        
        // Apply window function and copy to input buffer
        for i in 0..FFT_SIZE {
            self.input_buffer[i] = mono_samples[start_idx + i] * self.window[i];
        }
        
        // Perform FFT
        self.fft
            .process(&mut self.input_buffer, &mut self.output_buffer)
            .expect("FFT processing failed");
        
        // Convert to magnitude spectrum with logarithmic frequency scaling
        self.compute_log_spectrum(sample_rate);
        
        // Apply temporal smoothing
        self.apply_smoothing();
        
        &self.smoothed_spectrum
    }
    
    /// Compute logarithmically-scaled magnitude spectrum
    fn compute_log_spectrum(&mut self, sample_rate: u32) {
        let nyquist = sample_rate as f32 / 2.0;
        let min_freq = 20.0; // Human hearing lower bound
        let max_freq = nyquist.min(20000.0); // Human hearing upper bound
        
        // Clear spectrum
        self.spectrum.fill(0.0);
        
        // Logarithmic frequency bins
        for (bar_idx, bar_value) in self.spectrum.iter_mut().enumerate() {
            // Calculate frequency range for this bar (logarithmic scale)
            let freq_ratio = (bar_idx as f32 / NUM_BARS as f32).powf(2.0);
            let freq_start = min_freq + freq_ratio * (max_freq - min_freq);
            
            let next_freq_ratio = ((bar_idx + 1) as f32 / NUM_BARS as f32).powf(2.0);
            let freq_end = min_freq + next_freq_ratio * (max_freq - min_freq);
            
            // Convert frequencies to FFT bin indices
            let bin_start = (freq_start * FFT_SIZE as f32 / sample_rate as f32) as usize;
            let bin_end = (freq_end * FFT_SIZE as f32 / sample_rate as f32) as usize;
            
            // Average magnitude across bins in this frequency range
            let mut sum = 0.0;
            let mut count = 0;
            
            for bin_idx in bin_start..=bin_end.min(self.output_buffer.len() - 1) {
                let complex = self.output_buffer[bin_idx];
                let magnitude = (complex.re * complex.re + complex.im * complex.im).sqrt();
                sum += magnitude;
                count += 1;
            }
            
            if count > 0 {
                // Average and apply logarithmic scaling for better visualization
                let avg_magnitude = sum / count as f32;
                // Apply log scaling: log10(1 + magnitude) for better visual range
                *bar_value = (1.0 + avg_magnitude * 10.0).log10();
            }
        }
        
        // Normalize to 0-1 range
        if let Some(&max_val) = self.spectrum.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
            if max_val > 0.0 {
                for val in &mut self.spectrum {
                    *val /= max_val;
                }
            }
        }
    }
    
    /// Apply exponential moving average for temporal smoothing
    fn apply_smoothing(&mut self) {
        for i in 0..NUM_BARS {
            self.smoothed_spectrum[i] = SMOOTHING_FACTOR * self.smoothed_spectrum[i]
                + (1.0 - SMOOTHING_FACTOR) * self.spectrum[i];
        }
    }
    
    /// Get the number of frequency bars
    pub fn num_bars(&self) -> usize {
        NUM_BARS
    }
    
    /// Get the required number of samples for FFT
    pub fn required_samples(&self) -> usize {
        FFT_SIZE
    }
}

impl Default for FftAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
