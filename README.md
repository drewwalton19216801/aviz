# Audio Visualizer

A real-time audio visualizer built in Rust that displays frequency spectrum analysis of audio files.

## Features

- **Audio Playback**: Supports MP3 and WAV file formats
- **Real-time Visualization**: FFT-based frequency spectrum analysis with 128 frequency bars
- **Playback Controls**: Play, pause, stop, and volume control
- **Keyboard Shortcuts**: Press Space to play/pause
- **Color-coded Display**: Frequency bars change color based on magnitude (blue → cyan → green → yellow → red)
- **Smooth Animation**: 60 FPS updates with temporal smoothing

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```

## Usage

1. Click "Load File" to select an audio file (MP3 or WAV)
2. Use the playback controls to play, pause, or stop the audio
3. Adjust volume using the slider
4. Press Space to toggle play/pause
5. Watch the real-time frequency spectrum visualization

## Architecture

- **Audio Player** (`src/audio/player.rs`): Handles audio decoding and playback using rodio
- **FFT Analyzer** (`src/analysis/fft_analyzer.rs`): Performs real-time FFT analysis with Hann windowing
- **Visualizer** (`src/gui/visualizer.rs`): Renders the frequency spectrum with color gradients
- **Main Window** (`src/gui/main_window.rs`): GUI interface built with egui
- **App State** (`src/state/app_state.rs`): Manages application state and coordinates components

## Technical Details

- **FFT Size**: 4096 samples
- **Frequency Bars**: 128 bars with logarithmic frequency scaling
- **Frequency Range**: 20 Hz to 20 kHz (human hearing range)
- **Update Rate**: 60 Hz
- **Smoothing**: Exponential moving average with 0.7 factor

## Dependencies

- `rodio`: Audio playback and decoding
- `realfft`: Fast Fourier Transform
- `eframe/egui`: GUI framework
- `rfd`: Native file dialogs
- `anyhow`: Error handling
- `tracing`: Logging

## License

See LICENSE file for details.
