# 🎙️ VoxMagic: Gold Edition
<p align="center">
  <img src="VoxMagicLogo.png" width="220" alt="VoxMagic Logo">
</p>

# Elevate your speech to elite prose.

**VoxMagic** is a high-performance, native Windows application that transforms your spoken thoughts into polished, professional text instantly. Built with **Rust** and powered by **Groq's** world-class AI models, it offers a "magic" transcription experience with zero lag and near-perfect accuracy.

---

## ✨ Key Features

-   **🧠 The Magic Editor**: Don't just transcribe—refine. VoxMagic uses **Llama 3.3 70B** to automatically strip filler words ("um", "uh", "like"), fix complex grammar, and format your speech into perfect prose.
-   **⚡ Instant Auto-Paste**: Seamlessly inject refined text into any active window (ChatGPT, IDEs, Slacks, or Outlook) the microsecond you release the hotkey.
-   **🎹 Ultra-Responsive Hotkeys**: Optimized with native Windows API (`GetAsyncKeyState`) for hardware-level responsiveness. Hold **F8** or **Shift + Win** to begin your flow.
-   **🎙️ Studio-Grade Audio**: Integrated 16kHz downsampling engine and stereo-to-mono mixdown for maximum Whisper model compatibility.
-   **🔒 Privacy & Control**: Your API key is stored locally on your machine. No cloud tracking, just pure performance.
-   **💎 Premium Aesthetic**: A modern dark-mode UI with a pulsating ritualized visualizer that reacts to your voice.

## 🚀 Quick Start

1.  **Download**: Grab the latest `VoxMagic.exe` from the repo.
2.  **API Key**: Get your free API key from the [Groq Console](https://console.groq.com/keys).
3.  **Setup**:
    -   Launch **VoxMagic**.
    -   Click the **⚙️ Gear Icon**.
    -   Paste your **Groq API Key** and toggle **Always on Top**.
4.  **Commence Magic**:
    -   Focus on any text area.
    -   **Hold F8** and speak naturally.
    -   **Release** to watch your speech transform and paste automatically.

## 🛠️ Technical Architecture

-   **Core Engine**: Rust (Zero-cost abstractions & memory safety)
-   **UI Framework**: `egui` (Hardware-accelerated immediate mode GUI)
-   **Audio Pipeline**: `cpal` for low-latency capture & `hound` for WAV encoding.
-   **AI Inference**: 
    -   **Transcription**: Whisper V3 Turbo (Sub-second response)
    -   **Refinement**: Llama 3.3 70B (State-of-the-art formatting)
-   **Automation**: `enigo` for precise keyboard simulation.

## 🏗️ Building from Source

### Prerequisites
-   [Rust Toolchain](https://rustup.rs/) (Stable)
-   Windows 10/11

### Build Command
```powershell
# In the project root
cargo build --release
```
The optimized executable will be located in `target/release/VoxMagic.exe`.

## 📄 License

MIT © [Ashmil](https://github.com/ashmilgit15)

---
<p align="center"><i>Crafted for high-velocity thinkers.</i></p>
