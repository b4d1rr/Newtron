# 🚀 Newtron Roadmap (v0.1 → v0.9)

## 🧱 v0.1 — Core Skeleton
**Goal:** App exists + opens reliably

### Features
- Tauri app running (Rust + Vite)
- Global shortcut (Alt + N)
- Window show/hide toggle
- Basic command bar UI
- Input → handler → UI placeholder response

### Success Criteria
- Press shortcut → app appears instantly
- Type → something basic happens

---

## ⚙️ v0.2 — Command System
**Goal:** Make it feel like a real command bar

### Features
- Basic command parser
- Local commands (mocked is fine)
- Command history
- Focus handling improvements

### Success Criteria
- Spotlight-like basic behavior works

---

## 🧠 v0.3 — AI Bridge (Single Model)
**Goal:** First real AI integration

### Features
- Integrate ONE AI model (ChatGPT OR Claude OR Gemini)
- WebView injection OR API integration
- Response rendering in UI
- Loading state handling

### Success Criteria
- Type → send → AI responds inside Newtron

---

## 🧩 v0.4 — Architecture Cleanup
**Goal:** Stop technical debt early

### Features
- Clean module separation:
  - UI layer
  - Command layer
  - AI layer
- Event system (Rust ↔ frontend)
- Proper error handling

### Success Criteria
- Features can be added without breaking core flow

---

## 🔍 v0.5 — Multi-Command Routing
**Goal:** Smarter command behavior

### Features
- Command router system
- Local vs AI routing:
  - Local commands (calc, open apps)
  - AI commands
- Improved UX flow

### Success Criteria
- Feels like a real intelligent command bar

---

## 🌐 v0.6 — Multi-AI Support
**Goal:** 3 AI providers integrated

### Features
- ChatGPT integration
- Claude integration
- Gemini integration
- Settings menu for AI selection
- Unified response renderer

### Success Criteria
- Seamless switching between AI models

---

## 🔐 v0.7 — Login + Sessions
**Goal:** Real usability layer

### Features
- Visible WebView login flow
- Session persistence (cookies/storage)
- Per-AI session handling
- Secure storage (OS keychain if possible)

### Success Criteria
- User logs in once → stays logged in

---

## 🎨 v0.8 — Stability + UX Polish
**Goal:** Make it feel like real software

### Features
- Smooth animations
- Latency handling improvements
- Crash recovery
- Better focus + window behavior
- Tray integration (optional)
- Shortcut customization

### Success Criteria
- Feels like Raycast-level UX (lightweight + smooth)

---

## 🚀 v0.9 — Public Alpha
**Goal:** Usable public release

### Features
- Stable 3-AI system
- Reliable shortcut + window system
- Login system fully working
- Clean UI consistency pass
- Basic documentation (README upgrade)
- Bug-free core flows

### Success Criteria
- External users can install and actually use it

---

# 🧠 Notes
- Do NOT jump versions
- Focus on stability before features
- v0.3 → v0.6 is the hardest jump
- Perfection comes AFTER v0.6, not before