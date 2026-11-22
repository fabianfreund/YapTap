# YapTap: research and plan

> Hold a key, talk, let go. Clean text appears where your cursor is.
> Free and open source for macOS, Windows and Linux. **100% local: no cloud, no account, no word limits.**

YapTap is an open-source dictation app in the style of Wispr Flow, built for the [Appetizer](../../Appetizer-launcher) family next to [OneShot](../../OneShot).

**Ground rules**
1. **Cross-platform:** macOS, Windows and Linux from the start.
2. **No cloud, ever.** Speech recognition and text cleanup run on the device. There are no API keys, no hosted services and no telemetry. The network is used only to download models and check for updates, and an *Offline mode* switch turns both off.
3. Signed and notarized on macOS. Clean codebase, clean components, a sleek UI and good customization.

Research date: **2026-11-22**. Star counts are from GitHub on that day. Items marked **(?)** come from a single source or weren't verified, so they need a test before we rely on them.

---

## 1. What Wispr Flow actually is

It's cloud dictation followed by an LLM editing step, wrapped in a small floating bar that's always on screen. The promise: "talk instead of type, and it writes it properly." Wispr AI has raised over $300M (Series B in Aug 2026, at about a $2B valuation). It runs on Mac, Windows, iOS and Android.

| Layer | What it does | Why it feels good |
|---|---|---|
| **Trigger** | Hold `fn` (Mac) or `Ctrl+Win` (Windows) for push-to-talk. `fn+Space` or a double-tap starts hands-free mode. Shortcuts can be rebound (up to 3 keys, or a mouse button). | Nothing to click, works in any text field |
| **Flow Bar** | A small pill at bottom center. While listening it widens into a dark capsule with white level bars, ✗ and ✓ buttons, and a language picker. You can drag it to dock on any edge. | Shows it's ready without getting in the way. No live transcript: the text simply appears. |
| **AI cleanup** | Removes filler words, fixes grammar, turns spoken punctuation into real punctuation, and catches self-corrections ("at 2, actually 3" becomes "at 3"). Spoken lists become formatted lists. Levels: None, Light, Medium. ⌘Z undoes the cleanup. | You talk naturally and get text you could send |
| **Personalization** | A dictionary (including misspelling → correct pairs), snippets (spoken shortcuts that expand to longer text), and a writing style per kind of app (Formal, Casual, very casual, Excited). | It learns your names and jargon |
| **Command Mode / Transforms** | Select text, hold a key and say "make this shorter". Up to 8 saved prompts on hotkeys (`⌥1` Polish, `⌥2` Prompt Engineer). | Voice becomes an editing tool, not just an input method |
| **Context** | Sends the contents of the field you're typing in, the app and its category, the website domain, names visible on screen, and variable names in code editors. Screenshots are optional. | Names and code identifiers come out right |
| **Extras** | History (audio kept 14 days), stats (words per minute, streaks), a scratchpad, an iOS keyboard, an Android bubble, a meeting notetaker, a shared team dictionary | Habit loop, and a gradual expansion into a broader "AI workspace" |

**Speed target they publish:** under 700 ms (p99) from end of speech to text on screen: under 200 ms each for speech recognition, the LLM and the network. Independent tests measure about 400 ms for short dictations and about 900 ms for average ones.
Sources: [features](https://wisprflow.ai/features) · [help center](https://docs.wisprflow.ai) · [what's new](https://wisprflow.ai/whats-new) · [technical challenges](https://wisprflow.ai/research/technical-challenges)

### Where Wispr Flow is weak (our openings)

1. **Privacy and trust.** Everything goes through their cloud. Both data settings are *on* by default: training on your dictations, and storing them in the cloud. In 2025 there was a controversy over screenshots of the active window. An independent forensic report (Apr 2026) found a system-wide key logger hook, a 694 MB local database containing raw audio, hourly uploads even with sharing turned off, and telemetry sent to PostHog, Sentry, Segment and Datadog ([report](https://www.wensenwu.com/thoughts/wispr-flow-investigation)).
2. **Resources.** It's an Electron app with a Swift helper. Users report about 800 MB RAM and about 8% CPU while idle **(?)**.
3. **Online only.** No offline mode, multi-day outages in mid-2026, and "servers are busy" errors.
4. **Cost.** $144–180 a year. The free tier is capped at 2,000–5,000 words a week.
5. **Accuracy slips.** Words that flip the meaning get dropped ("never merge" became "merge"), and the cleanup sometimes over-edits.
6. **Gaps.** Writing styles are English only, you can't mix languages within a sentence, there's no Linux version, and snippets can't contain variables.

**YapTap's pitch:** *as fast as Wispr Flow or faster, works offline, nothing leaves your computer, runs on Mac, Windows and Linux, open source, free forever.*

---

## 2. The landscape: we're not the first

Local, open-source dictation is a crowded field in 2026. YapTap has to win on **polish, reliable text insertion, good cleanup that works out of the box, and a permissive license**, not on "free and local" alone.

| App | License | ★ | Stack | Platforms | Local speech engine | Local cleanup | Notes |
|---|---|---|---|---|---|---|---|
| [Handy](https://github.com/cjpais/Handy) | **MIT** | 39.6k | Tauri 2 + Rust | Mac, Windows, Linux | transcribe.cpp (Whisper, Parakeet) | Apple Intelligence; cloud APIs | Biggest by far. Deliberately minimal, basic UI. Top issue (94 comments): it pastes the old clipboard instead of the transcript. **Closest reference for our stack.** |
| [Hex (anomalyco)](https://github.com/anomalyco/hex) | **MIT** | 8.0k | Rust core + native shells | Mac, Windows, Linux | transcribe.cpp fork | Script pipeline | Strongest MIT competitor. Per-app output pipeline, command line, MCP server. |
| [OpenWhispr](https://github.com/OpenWhispr/openwhispr) | **MIT** | 12.0k | Electron + React | Mac, Windows, Linux | whisper.cpp, sherpa-onnx | llama.cpp | Feature creep (notes, meetings, teams, its own cloud). Heavy. |
| [FluidVoice](https://github.com/altic-dev/FluidVoice) | GPL-3 | 13.2k | Swift | Mac (Windows and Linux in beta) | Own models, Parakeet, Nemotron | **Closed-source add-on** | Overlay that wraps the MacBook notch. Reports of high idle CPU and slow cleanup. |
| [VoiceInk](https://github.com/Beingpax/VoiceInk) | GPL-3 | 7.2k | Swift | Mac | whisper.cpp, FluidAudio | Ollama, cloud APIs | Good per-app "Power Mode". Takes **no outside contributions**; the signed build costs $25. |
| [TypeWhisper](https://github.com/TypeWhisper/typewhisper-mac) | GPL-3 | 1.9k | Swift | Mac | FluidAudio, Apple speech | MLX (Gemma 4, Qwen3.5) | Closest to our cleanup idea, but GPL and Mac only |
| [Vibe](https://github.com/thewh1teagle/vibe) | MIT | 8.2k | Tauri | Mac, Windows, Linux | whisper.cpp, Parakeet | Ollama | Transcribes files, not dictation. Good reference for cross-platform GPU builds. |
| Superwhisper / Spokenly / MacWhisper | closed | – | native | varies | Whisper, Parakeet | partial | Spokenly is free locally and has a "Local Only Mode" |
| Aqua, Willow | closed | – | – | Mac, Windows | none (cloud) | – | Subscriptions, no offline mode |
| Apple / Windows built-in dictation | system | – | – | – | yes | Apple adds Gemini-based fixes in iOS 27 | Free, weak vocabulary, no styles |

### What every serious dictation app has (v1 needs all of it)

- Push-to-talk **and** toggle, ideally on one key: hold to talk, tap or double-tap to start hands-free.
- Paste at the cursor and restore the clipboard afterwards. A typing fallback. "Paste last transcript."
- Parakeet v3 as the fast default model and Whisper turbo for more languages, a model manager, and voice activity detection (automatic silence trimming).
- Custom vocabulary and text replacements.
- Optional cleanup with a style per app.
- A floating pill with a sound-level meter.
- Searchable history with a retention setting.
- Pausing or muting music while recording.
- Auto-update, signed builds, and a package manager install (Homebrew, winget).

### Gaps nobody fills well (our opportunity)

1. **Open, local cleanup that works out of the box on every platform.** FluidVoice's is closed and Mac only. Handy uses Apple Intelligence on Mac and otherwise *cloud*. OpenWhispr needs configuration. → We embed llama.cpp with a small open model, installed in one click, identical on all three platforms.
2. **Text insertion that actually lands.** Paste bugs are the #1 issue for Handy, VoiceInk, OpenWhispr and FluidVoice. → A per-app insertion strategy table, a check that the text landed, a clipboard restore that keeps images and formatting, and a public list of tested apps.
3. **A polished cross-platform UI.** Handy is functional but plain. The polished apps are Mac-only Swift apps. → An app with OneShot-level design on all three platforms.
4. **Strictly no cloud.** Every other open app offers cloud APIs. Making "no cloud" a hard rule is a clear, auditable promise.
5. **Light footprint.** → A small overlay, no hidden heavy windows, the model unloaded after idle time, and a short audio buffer that's always recording so the first syllable is never lost.
6. **Transparency.** → The raw and the cleaned text side by side, and one key to undo the cleanup.
7. **Learning from your own corrections**, plus importing names (contacts, a project's identifiers). No open app does this today.
8. **German and other languages handled well.** Wispr's writing styles are English only, and Parakeet can't be locked to one language (see §3).

---

## 3. Technology choices (all local)

### Speech-to-text: one engine on all platforms

**[transcribe.cpp](https://github.com/handy-computer/transcribe.cpp)** (MIT, 2.2k ★, built by the Handy team, Rust crate `transcribe-cpp`).
- Runs 16+ model families in one file format: Whisper, Parakeet (v3), Canary, Nemotron 3.5 streaming, Qwen3-ASR, Moonshine, Cohere, Granite, plus Silero voice activity detection and language detection.
- Uses the GPU on each platform: **Metal** on macOS, **Vulkan** on Windows and Linux, CUDA optional, with a CPU fallback.
- Handy ships on it on all three platforms, and VoiceInk has started using it too.

| Model | Size | German | Notes |
|---|---|---|---|
| **Parakeet TDT 0.6B v3** (default) | ~0.5–0.65 GB | ✅ FLEURS **5.9% word error rate** | 25 European languages, punctuation and capitals included. Very fast: about 200× real time on the Apple Neural Engine. Expect somewhat slower on GPU through ggml, so measure it. The model's license (CC-BY-4.0) requires credit. |
| **Whisper large-v3-turbo** | ~0.6 GB (q5) – 1.6 GB | ✅ the language **can be locked** | 99 languages. Light vocabulary hints through a prompt. MIT. |
| Whisper small / base | 0.2–0.5 GB | ✅ | For low-end PCs and machines without a GPU |
| Nemotron 3.5 Streaming Multilingual 0.6B | 0.6B | ✅ 8.3–9.8% error rate | Real streaming, for a later **live preview** in the pill |
| Qwen3-ASR 0.6B | ~650 MB | ✅ | Apache-2.0, accepts free-form context (names, jargon). Worth watching. |

**Decision:** default to **Parakeet v3**, with **Whisper turbo plus a locked language** as an option and Whisper small for weak hardware. Transcribe the whole recording when you let go of the key; at this speed it adds almost no wait and is more accurate than live streaming. A streaming preview can come later.

⚠️ **Parakeet v3 has no language lock.** There are reports of it quietly *translating* non-English speech into English **(?)**. Test German and mixed German/English early. If it drifts, use Whisper turbo for "always German."

*Optional Mac speed-up for later:* Apple's own SpeechTranscriber (macOS 26, no download), or FluidAudio on the Neural Engine, called through a small Swift bridge. Handy already does something similar for Apple Intelligence. Only worth it if measurements show a real gain.

### Voice activity detection and audio
- **Silero voice activity detection v6.2**, included in transcribe.cpp, about 2 MB. It trims silence, which also stops Whisper from inventing text during quiet parts. It also stops hands-free mode automatically and throws away empty recordings.
- **Capture:** [`handy-recorder`](https://github.com/handy-computer/handy-recorder) (MIT, cross-platform, built on cpal), or `cpal` directly. Audio is converted to **16 kHz mono**. The mic stays open with a **~300 ms rolling buffer** so the first syllable is never lost (VoiceInk loses about 1 s), plus a mic priority list.
- **Pause or mute media while recording:**
  - Mac: [mediaremote-adapter](https://github.com/ungive/mediaremote-adapter), falling back to muting the output.
  - Windows: the system media controls API (`GlobalSystemMediaTransportControlsSessionManager`), falling back to muting the endpoint.
  - Linux: MPRIS over D-Bus.
  - Only resume or unmute what *we* paused.

### Local LLM (cleanup, styles, command mode)
- **Off by default.** Plain transcription plus rule-based formatting (spoken punctuation, replacements, snippets) is instant and never makes things up.
- **"Polish" when turned on:** embedded **llama.cpp** via [`llama-cpp-2`](https://github.com/utilityai/llama-cpp-rs) (Apache-2.0), with the same GPU backends (Metal, Vulkan, CUDA, CPU) and the same model manager as speech.
  - Default model: **Qwen3.5 2B** at 4-bit, under 2 GB, Apache-2.0. Options: Qwen3.5 0.8B for weak machines, Qwen3.5 4B or Gemma 4 E2B for quality.
  - Expected speed is about 0.5–0.8 s per cleanup on an M-series Pro **(?)**. Needs measuring on weak Windows laptops.
- **Mac bonus:** Apple's on-device Foundation Models (macOS 26+, free, no download, still local) through a Swift bridge, picked automatically when available.
- **No cloud providers, no keys.** Supporting Ollama or LM Studio would only be considered later, and only on `localhost`.
- Guardrails:
  - A strict system prompt: only fix the text, never answer it.
  - Output capped at about 1.5× the input length.
  - Fall back to the raw text after a timeout (about 1.5 s).
  - The raw text is always kept in history.

### Platform integration

| Need | macOS | Windows | Linux |
|---|---|---|---|
| **Global hotkeys** (including modifier-only and hold) | [`handy-keys`](https://github.com/handy-computer/handy-keys) (MIT): supports **fn/Globe**, modifier-only keys, left and right modifier keys, mouse buttons | `handy-keys`: low-level keyboard hook, no permission needed | `handy-keys`: reads keyboard input directly, works on X11 **and Wayland**. Needs a udev `uaccess` rule shipped in the .deb/.rpm. Fallback: `yaptap --toggle` bound to a shortcut in your desktop's own settings. |
| Default shortcut | hold `fn` (or right ⌥) | hold `Ctrl+Win` (or right Alt) | hold right Ctrl (configurable) |
| **Insert text** | 1) clipboard + simulated ⌘V, then restore the clipboard 2) Accessibility API set-text for native apps 3) simulated typing for terminals | 1) clipboard + simulated Ctrl+V, then restore (`SendInput` via enigo) 2) simulated typing for terminals and remote desktops | X11: clipboard + Ctrl+V or simulated typing (xdotool-style via enigo). Wayland: `wtype` / `ydotool`, or the desktop portal's remote-desktop interface **(?)** |
| Read selected text and current app | Accessibility API, frontmost app | UI Automation, foreground window | X11 primary selection, active window. Wayland: limited |
| **Overlay that doesn't steal focus** | [`tauri-nspanel`](https://github.com/ahkohd/tauri-nspanel) (Apache-2.0): non-activating panel on all Spaces, including over full-screen apps | always-on-top tool window that never activates (`WS_EX_NOACTIVATE \| WS_EX_TOOLWINDOW \| WS_EX_TOPMOST`), click-through when idle | X11: always-on-top utility window. Wayland: layer-shell (`gtk-layer-shell`). Handy has known Wayland focus bugs. |
| Permissions | Microphone, Accessibility, Input Monitoring (only for fn or modifier-only shortcuts) via `tauri-plugin-macos-permissions` | Microphone privacy setting | `input` device access (udev rule), PipeWire/Pulse mic |
| Tray / menu bar | Tauri tray, no Dock icon | system tray | tray via AppIndicator / StatusNotifier |

- **Mac:** users need to set "Press 🌐 key to → Do Nothing" (System Settings → Keyboard), or fn opens the emoji picker. Onboarding checks this and links straight to the setting.
- **Mac:** not sandboxed (Accessibility control and simulated keystrokes don't work inside the sandbox). Developer ID signed outside the App Store, like OneShot and the launcher.
- **Linux on Wayland is the hardest platform** (shortcuts, typing text in, overlay). Plan: X11 and KDE/GNOME Wayland "best effort" at launch, with a compatibility table in the README.

---

## 4. Stack decision: Tauri 2 + Rust + React

| | **Tauri 2 + Rust** | Electron (OneShot's stack) | Native per platform |
|---|---|---|---|
| Cross-platform from one codebase | ✅ | ✅ | ❌ (2–3 apps) |
| Installer size | **~15 MB** (Handy) | 100 MB+ | small |
| Idle memory | low (Rust plus the system webview) | 150–300 MB | lowest |
| Local AI engines | **Rust crates directly** (transcribe-cpp, llama-cpp-2) | needs native modules or a helper | direct |
| Overlay and hotkeys | proven by Handy (tauri-nspanel, handy-keys, enigo) | hardest | best |
| Fits the family | **same stack as the Appetizer launcher** (Tauri 2, React 19, Vite, zustand, tauri-plugin-updater, release-mac.sh) | same as OneShot | – |

**Decision: Tauri 2 + Rust (backend: audio, models, hotkeys, insertion) + React/TypeScript (UI)**, the same as the launcher. Handy proves the architecture works for exactly this kind of product, and its building blocks are MIT, so we can use them.

**Why not fork Handy?** Its scope is deliberately minimal, its UI and onboarding would be rewritten anyway, and we want a cleaner component structure. Instead we **reuse its MIT crates** (`transcribe-cpp`, `handy-keys`, `handy-recorder`) and learn from its handling of each platform's quirks, crediting it where we copy code.

Ways to keep idle use low:
- Create the settings and history windows only when opened, and destroy them on close.
- The overlay is one small, plain HTML/CSS window. Its level-meter animation runs only while recording.
- Load models on first use and unload them after N minutes idle (configurable).

---

## 5. UI and design

The look is OneShot's calm, Apple-like style. Because this is a web stack, **we can take OneShot's `tokens.css` almost as is**: glass surfaces, hairline borders, soft shadows, one accent color, and automatic light and dark mode. The pill should feel alive while you speak and stay out of sight otherwise. On Windows and Linux it uses the same design, with system fonts (Segoe UI Variable, Inter/Cantarell) and platform-correct window controls.

### The pill (overlay)

```
 idle (optional)     listening                         processing           done
   ╭──╮          ╭──────────────────────────╮      ╭────────────╮      ╭─────────╮
   ╰──╯          │ ✕   ▂▅▇▅▃▂▅▇▆▃   0:04  ✓ │      │  ◌ ◌ ◌     │      │  ✓      │ → fades
                 ╰──────────────────────────╯      ╰────────────╯      ╰─────────╯
```

- **States:**
  - hidden or idle
  - **listening:** live level bars that follow your voice, ✕ cancel, ✓ stop, a timer
  - **locked:** hands-free mode, shown by a small lock icon
  - **processing:** the bars shrink into a soft shimmer or pulsing dots
  - **inserted:** a short ✓, then the pill fades out
  - **error:** red tint plus a short message ("No mic access", "Nothing heard", "Couldn't paste: copied to clipboard")
- **Motion:** springy width changes of about 0.35 s, with OneShot's easing curve `--ease: cubic-bezier(.2,.8,.2,1)`. The bars animate at 60 fps with smoothing so they never jitter. Respects "reduce motion".
- **Placement:** bottom center above the Dock/taskbar by default. Optional "notch mode" on MacBooks. Draggable with snap zones; the position is remembered per screen. It follows the screen your mouse is on.
- **Look:** a dark glass capsule with a hairline edge and a soft floating shadow. The bars are in the accent color or white.
- **Sounds:** soft start and stop ticks, short and quiet, our own. Can be turned off or replaced.
- **Live preview (later, with streaming):** the last few words fade in above the pill.

### Windows (UI)
- **Tray / menu bar menu:** status, last transcript (click to copy), mode switches, the active model, "Paste last", Settings, Quit.
- **Settings:** one window with a sidebar: General · Shortcuts · Models · Polish (AI) · Styles per app · Dictionary · Snippets · History · Appearance · Privacy · About.
- **Onboarding:** five steps, like OneShot's welcome screen:
  1. Welcome
  2. Microphone, with a live test
  3. Permissions for the current platform
  4. Pick a shortcut (includes the Globe-key check on Mac)
  5. Model download with progress, while you **try it out** in a practice field
- **History:** a searchable list grouped by day. Raw and polished text side by side, copy, run the cleanup again, delete. Retention: forever, 7 days, 24 h, or never save.

### Customization (the "good customization" goal)

| Area | Options |
|---|---|
| Shortcuts | Any key combination, fn/Globe (Mac), right ⌥/Alt, right ⌘/Ctrl, a mouse button. Hold, toggle, or both. Separate shortcuts for Command mode, Paste last and Cancel. |
| Models | Parakeet v3 / Whisper turbo / Whisper small. Language: automatic or locked. GPU or CPU. Model folder location. Delete or re-download. |
| Polish | Off / Light / Medium / your own prompt. Choice of local model. Timeout. |
| Styles per app | Rules by app (and by website on Mac). Style (formal / casual / all-lowercase chat), polish on or off, how text is inserted, language. Comes with presets: chat apps casual, mail formal, terminals and code editors raw text typed in. |
| Dictionary | Words to recognize better, "heard → write" replacements, import and export as CSV. |
| Snippets | A spoken phrase that expands into text, with variables (`{date}`, `{clipboard}`, `{selection}`), which Wispr doesn't have. |
| Overlay | Position (bottom / top / notch / custom), size (mini / regular), show when idle, accent color, bar style (bars / wave / dot), sounds, reduce motion. |
| Audio | Mic priority list, "prefer built-in mic", pause or mute media, stop automatically after N seconds of silence. |
| Privacy | History retention, save audio (off by default), **Offline mode** (no network at all), what context may be read (selected text / app). |
| Power users | A script that receives the text and returns a changed version before it's pasted, a command line (`yaptap --toggle`, `yaptap transcribe file.wav`), a URL scheme (`yaptap://toggle`). |

---

## 6. Architecture

```
YapTap/
├─ src/                         React + TypeScript (Vite), same setup as the launcher
│  ├─ styles/tokens.css         shared design tokens (ported from OneShot)
│  ├─ components/               Pill, LevelBars, GlassPanel, SettingsRow, Toggle, Segmented,
│  │                            ShortcutRecorder, PermissionCard, ModelCard, ProgressRing …
│  ├─ windows/                  overlay/, settings/, onboarding/, history/  (one entry per window)
│  ├─ store/                    zustand stores mirrored from Rust settings
│  └─ lib/ipc.ts                typed commands/events (tauri-specta generated)
└─ src-tauri/src/
   ├─ session.rs                state machine: idle → recording → transcribing → polishing → inserting → done/error
   ├─ audio/                    capture (handy-recorder/cpal, 16k mono, pre-roll), level meter, media pause per OS
   ├─ hotkeys.rs                handy-keys, hold vs tap logic
   ├─ stt/                      trait SpeechEngine → TranscribeCpp (Parakeet, Whisper, …); VAD
   ├─ polish/                   trait Polisher → LlamaCpp, AppleFM (macOS); prompts
   ├─ text/                     pure rule pipeline: spoken punctuation → replacements → snippets → style
   ├─ insert/                   trait Inserter → Paste, Type, Accessibility (per OS); per-app strategy table; clipboard snapshot/restore
   ├─ context/                  current app, selected text (per OS)
   ├─ models/                   catalog.json, downloader (pinned Hugging Face commit + SHA-256), storage
   ├─ platform/{macos,windows,linux}.rs   all OS-specific code lives here, nowhere else
   └─ store/                    settings.json, history.sqlite (rusqlite)
```

- **Every engine and every OS-specific part sits behind a trait**, so the core logic can be tested without hardware and without a particular OS.
- **The text pipeline is pure functions**, which makes self-corrections, lists, punctuation and snippets easy to unit-test.
- **Typed IPC** between Rust and the UI via `tauri-specta` (Handy does this too), so the frontend can't drift from the backend.
- **No network code outside `models/` and the updater.** Easy to audit, and Offline mode blocks both.
- **No telemetry.** At most an opt-in local log file the user can share.

---

## 7. Shipping

- **Models:** downloaded on first run, never bundled. Hugging Face URLs pinned to a fixed commit, checked with SHA-256, resumable. Stored in the app data folder: `~/Library/Application Support/app.yaptap/`, `%APPDATA%\app.yaptap\`, or `~/.local/share/app.yaptap/`. Manual import for users behind a proxy or offline.
- **macOS:** Developer ID (Digital Vibes GmbH, team 8HFX6YPXXN, same as OneShot and the launcher), hardened runtime, not sandboxed, notarized and stapled.
  - Reuse the launcher's `scripts/release-mac.sh` approach (Tauri bundler, `notarytool` keychain profile).
  - Entitlement: `com.apple.security.device.audio-input`.
  - Info.plist: `NSMicrophoneUsageDescription`, `LSUIElement`.
  - Universal or arm64 DMG. The Metal backend is compiled in.
- **Windows:** NSIS `.exe` for x64, plus ARM64 if Vulkan or CPU works there, built by GitHub Actions like OneShot. Vulkan backend with a CPU fallback.
  - **Shipped unsigned for now (decided 2026-11-22)**, the same as OneShot's Windows build. It's a free app; we can revisit signing later.
  - What that means for users: SmartScreen shows "Windows protected your PC" until the download builds up reputation. The README and the release notes explain **More info → Run anyway**.
  - Because YapTap listens to the keyboard, some antivirus tools may flag it as suspicious. If that happens, submit the build to Microsoft's [file submission portal](https://www.microsoft.com/wdsi/filesubmission) as a false positive, and keep the build reproducible from the public CI.
  - If this becomes a real problem later, Azure Trusted Signing is the cheapest fix.
- **Linux:** AppImage, `.deb` and `.rpm`, unsigned. The `.deb`/`.rpm` include the udev `uaccess` rule for hotkeys; the AppImage shows a one-time setup hint.
- **Updates:** `tauri-plugin-updater` with a minisign key and `latest.json`, the same as the launcher. Agree with the launcher who owns updates so they don't fight; the launcher already updates the apps it installed.
- **Appetizer listing:** add `appetizer.json` (name, tagline, accent color, `build/icon.png`, screenshots, `"platforms": ["macos","windows","linux"]`), then `pnpm catalog add fabianfreund/YapTap`.
- **Licenses:** YapTap is MIT.
  - Credit NVIDIA Parakeet (CC-BY-4.0) in About and in the README.
  - Keep the license and notice files of the Apache-2.0 dependencies (llama-cpp-2, tauri-nspanel).
  - Credit Handy's MIT crates.
  - **Don't copy code from VoiceInk, FluidVoice or TypeWhisper (GPL) or Epicenter (AGPL).** Reading them for ideas is fine.

---

## 8. Roadmap

**MVP (v0.1): "hold, talk, paste" done perfectly, on all three platforms**
- Tray app, onboarding with permission checks for each platform, mic test, model download
- Hold to talk + tap for hands-free + custom shortcut, Esc to cancel
- Parakeet v3 via transcribe.cpp (Metal / Vulkan / CPU), Silero voice activity detection, the rolling start buffer
- Paste insertion with clipboard restore, typing fallback, "Paste last"
- The pill with all its states, sounds, reduce motion
- Spoken punctuation, dictionary replacements, history (raw text)
- Signed and notarized on Mac, auto-update, listed in Appetizer

**v0.2: polish and personalization**
- Local Polish (llama.cpp with Qwen3.5 2B, Apple Foundation Models on Mac) with a raw-vs-polished view and undo
- Self-correction and list formatting: rules first, the LLM when Polish is on
- Styles and insertion methods per app, snippets with variables
- Whisper turbo with a language lock, mic priority list, pausing media on each OS

**v0.3: deep**
- Command mode (voice edits the selected text) and shortcut-driven transforms
- Streaming live preview (Nemotron), notch mode on Mac
- Learning vocabulary from your corrections, importing contacts and project terms
- Script hook, command line, URL scheme, a local MCP server for coding agents
- Stats (words per minute, words saved), kept on the device only

**Later:** iOS and Android keyboards, meeting notes.

---

## 9. Validate before committing (quick tests)

1. Parakeet v3 via transcribe.cpp on real **German** and mixed German/English speech: does it drift into English? How fast is Metal vs Vulkan vs CPU?
2. Speed on a **base M1 with 8 GB** and on a **mid-range Windows laptop without a dedicated GPU** (Vulkan on integrated graphics vs CPU).
3. Polish speed with Qwen3.5 0.8B / 2B on the same two machines. Is Apple Foundation Models faster on Mac?
4. `handy-keys` fn behavior on Mac when the Globe setting isn't "Do Nothing", and on external keyboards.
5. Insertion test across apps: browser text areas, Slack, Notion, VS Code/Cursor, Terminal/iTerm/Ghostty/Windows Terminal, the Claude Code terminal UI, Google Docs, Word. Password fields must be refused.
6. Windows: does Microsoft Defender or another common antivirus flag the unsigned build because it hooks the keyboard?
7. Linux: GNOME and KDE on Wayland for shortcuts, typing text in, and the overlay.
8. Idle memory and CPU of the Tauri app with the overlay hidden, with the model loaded and unloaded.

## 10. Decisions for v0.1 (2026-11-22)

- **Signing:** macOS signed and notarized. Windows and Linux ship unsigned for now.
- **Defaults chosen to get a first version running** (all easy to change):
  - Minimum macOS 13. Wayland is best effort, with `yaptap --toggle` as the fallback shortcut.
  - Bundle id `app.yaptap`, accent color `#7B61FF`.
  - Windows and Linux run speech recognition on the **CPU** for now. Vulkan through `dynamic-backends` is the next step.
  - The mic opens per recording, not kept warm, so the system "mic in use" indicator is only on while dictating.
  - No in-app updater yet. The Appetizer launcher keeps installed copies up to date.

### Still open
- Pre-roll buffer ("keep the mic ready") as an opt-in setting.
- Updater ownership once the app has its own updater.
- Local "Polish" (llama.cpp) in v0.2.
