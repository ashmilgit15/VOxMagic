//! Configuration module for Speech-to-Text application
//! Note: Users must provide their own API Key via the app settings.

pub const GROQ_API_URL: &str = "https://api.groq.com/openai/v1/audio/transcriptions";
pub const WHISPER_MODEL: &str = "whisper-large-v3-turbo";

/// Audio Configuration
pub const CHANNELS: u16 = 1;
pub const BITS_PER_SAMPLE: u16 = 16;
