//! Real-time audio spectrum visualizer with color-coded frequency bars.

use eframe::egui;

/// Visual representation of audio frequency spectrum.
///
/// Displays frequency bars with color gradients based on magnitude,
/// ranging from blue (low) to red (high).
pub struct Visualizer {
    spectrum_data: Vec<f32>,
}

impl Visualizer {
    pub fn new() -> Self {
        Self {
            spectrum_data: vec![0.0; 128],
        }
    }
    
    /// Update the spectrum data for visualization
    pub fn update_spectrum(&mut self, spectrum: &[f32]) {
        self.spectrum_data.clear();
        self.spectrum_data.extend_from_slice(spectrum);
    }
    
    /// Render the visualizer
    pub fn ui(&self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();
        let desired_height = 300.0;
        let height = available_size.y.min(desired_height);
        
        let (response, painter) = ui.allocate_painter(
            egui::vec2(available_size.x, height),
            egui::Sense::hover(),
        );
        
        let rect = response.rect;
        
        // Draw background
        painter.rect_filled(
            rect,
            4.0,
            egui::Color32::from_rgb(20, 20, 30),
        );
        
        // Check if we have any actual audio data (non-zero values)
        let has_audio = self.spectrum_data.iter().any(|&v| v > 0.001);
        
        if !has_audio {
            // Draw "No audio playing" message
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "No audio playing",
                egui::FontId::proportional(16.0),
                egui::Color32::from_rgb(100, 100, 120),
            );
            return;
        }
        
        // Calculate bar dimensions
        let num_bars = self.spectrum_data.len();
        let bar_spacing = 2.0;
        let total_spacing = bar_spacing * (num_bars - 1) as f32;
        let available_width = rect.width() - 20.0; // 10px padding on each side
        let bar_width = (available_width - total_spacing) / num_bars as f32;
        
        let max_bar_height = rect.height() - 20.0; // 10px padding top and bottom
        
        // Draw frequency bars
        for (i, &magnitude) in self.spectrum_data.iter().enumerate() {
            let x = rect.left() + 10.0 + i as f32 * (bar_width + bar_spacing);
            let bar_height = (magnitude * max_bar_height).max(2.0);
            let y = rect.bottom() - 10.0 - bar_height;
            
            let bar_rect = egui::Rect::from_min_size(
                egui::pos2(x, y),
                egui::vec2(bar_width, bar_height),
            );
            
            // Color gradient based on magnitude: blue -> cyan -> green -> yellow -> red
            let color = Self::magnitude_to_color(magnitude);
            
            painter.rect_filled(
                bar_rect,
                1.0,
                color,
            );
        }
        
        // Draw frequency labels
        let label_y = rect.bottom() - 5.0;
        let label_color = egui::Color32::from_rgb(150, 150, 160);
        let font_id = egui::FontId::proportional(10.0);
        
        // Low frequency label (left)
        painter.text(
            egui::pos2(rect.left() + 10.0, label_y),
            egui::Align2::LEFT_BOTTOM,
            "20Hz",
            font_id.clone(),
            label_color,
        );
        
        // High frequency label (right)
        painter.text(
            egui::pos2(rect.right() - 10.0, label_y),
            egui::Align2::RIGHT_BOTTOM,
            "20kHz",
            font_id,
            label_color,
        );
    }
    
    /// Convert magnitude (0.0-1.0) to a color gradient
    fn magnitude_to_color(magnitude: f32) -> egui::Color32 {
        let mag = magnitude.clamp(0.0, 1.0);
        
        // Color gradient: blue -> cyan -> green -> yellow -> red
        if mag < 0.2 {
            // Blue to Cyan
            let t = mag / 0.2;
            egui::Color32::from_rgb(
                0,
                (t * 100.0) as u8,
                (100.0 + t * 155.0) as u8,
            )
        } else if mag < 0.4 {
            // Cyan to Green
            let t = (mag - 0.2) / 0.2;
            egui::Color32::from_rgb(
                0,
                (100.0 + t * 155.0) as u8,
                (255.0 - t * 155.0) as u8,
            )
        } else if mag < 0.6 {
            // Green to Yellow
            let t = (mag - 0.4) / 0.2;
            egui::Color32::from_rgb(
                (t * 255.0) as u8,
                255,
                (100.0 - t * 100.0) as u8,
            )
        } else if mag < 0.8 {
            // Yellow to Orange
            let t = (mag - 0.6) / 0.2;
            egui::Color32::from_rgb(
                255,
                (255.0 - t * 100.0) as u8,
                0,
            )
        } else {
            // Orange to Red
            let t = (mag - 0.8) / 0.2;
            egui::Color32::from_rgb(
                255,
                (155.0 - t * 155.0) as u8,
                0,
            )
        }
    }
}

impl Default for Visualizer {
    fn default() -> Self {
        Self::new()
    }
}
