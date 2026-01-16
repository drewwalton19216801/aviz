use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("Failed to load audio file: {0}")]
    LoadError(String),
    
    #[error("Failed to decode audio: {0}")]
    DecodeError(String),
    
    #[error("Playback error: {0}")]
    PlaybackError(String),
    
    #[error("No audio file loaded")]
    NoFileLoaded,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AudioError>;
