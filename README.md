# 🎙️ Wispr Flow Pro
<p align="center">
  <img src="VoxMagicLogo.png" width="200" alt="VoxMagic Logo">
</p>

**Write with your voice, at the speed of thought.**

Wispr Flow Pro is a high-performance native Windows application that transforms your speech into polished, professional text instantly. Built with Rust and powered by Groq's world-class AI models, it offers a "magic" transcription experience with zero lag.

## ✨ Key Features

-   **🧠 Magic Editor**: Automatically removes filler words ("um", "uh"), fixes grammar, and formats your speech into perfect prose using Llama 3.3 70B.
-   **⚡ Instant Auto-Paste**: Transcribed text is instantly pasted into any active window (ChatGPT, Notepad, Email, etc.) the moment you release the hotkey.
-   **🎹 Hardware-Level Hotkeys**: Uses native Windows API polling for ultra-responsive global hotkey detection.
-   **🎙️ Studio-Grade Audio**: High-fidelity 16kHz downsampling engine for crystal-clear voice extraction.
-   **⚙️ Persistent Settings**: Save your API key and preferences locally.
-   **🚫 No Echo**: Built-in stereo-to-mono mixing eliminates "doubling" text issues.

## 🚀 Quick Start

1.  **Download**: Get the latest `WisprFlowPro.exe` from the [Releases](https://github.com/YOUR_GITHUB_USER/YOUR_REPO/releases) page.
2.  **API Key**: Create a free account at the [Groq Console](https://console.groq.com/keys) and generate an API key.
3.  **Configure**: Launch the app, click the ⚙️ gear icon, and paste your API key.
4.  **Flow**:
    -   Click into any text field.
    -   **Hold F8** (or Shift+Win) and speak naturally.
    -   **Release** to watch the magic happen!

## 🛠️ Technical Stack

-   **Language**: Rust (Systems-level performance)
-   **GUI**: egui/eframe (Immediate mode UI)
-   **Audio**: cpal & hound (Native audio capture & processing)
-   **AI Inference**: Groq API (Whisper V3 Turbo + Llama 3.3 70B)
-   **Keyboard**: Native Windows `GetAsyncKeyState`

## 🏗️ Building from Source

### Prerequisites
-   [Rust Toolchain](https://rustup.rs/)
-   Windows 10/11

### Build Command
```powershell
cd speech_to_text
cargo build --release
```
The executable will be located in `target/release/WisprFlowPro.exe`.

## 📄 License

MIT License - feel free to use and contribute!

---
*Generated with 💡 by Wispr Flow Pro Team*
