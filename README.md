# TrackLog

TrackLog is a lightweight desktop application built with Tauri for managing and organizing song-related data locally.

This project combines a modern web frontend with a Rust backend, delivering a fast, secure, and minimal desktop experience.

---

## 🚀 Tech Stack

* **Frontend**: HTML / JavaScript (Vite-based)
* **Backend**: Rust
* **Framework**: Tauri
* **Package Manager**: npm

---

## 📦 Project Structure

```
TrackLog/
│
├── src/                 # Frontend source code
├── src-tauri/          # Tauri (Rust backend)
│   ├── src/            # Rust source
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── package.json
├── package-lock.json
└── .gitignore
```

---

## ⚙️ Requirements

Make sure the following are installed:

* Node.js (LTS)
* Rust (stable toolchain via rustup)
* Visual Studio Build Tools (Windows)
* WebView2 (usually already installed on Windows)

---

## 🛠️ Setup

Clone the repository and install dependencies:

```
git clone https://github.com/YOUR_USERNAME/tracklog.git
cd tracklog
npm install
```

---

## 🧪 Development

Run the app in development mode:

```
npm run tauri dev
```

---

## 🏗️ Build

Create a production build:

```
npm run tauri build
```

The generated application will be available under:

```
src-tauri/target/release/
```

and installer bundles (if enabled) under:

```
src-tauri/target/release/bundle/
```

---

## 📌 Current Status

This is an early version of TrackLog.

Features and structure are still evolving, and the focus is currently on:

* Establishing the application foundation
* Setting up Tauri integration
* Preparing for future feature development

---

## 🧭 Roadmap (planned)

* Song catalog management
* Metadata editing and organization
* Search and filtering capabilities
* UI improvements
* Data persistence enhancements

---

## 📄 License

This project is currently private / not licensed for distribution.

---

## 👤 Author

Stefano Mingoli

---
