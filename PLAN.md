# AetherCast — Project Plan

> **AI Hint:** Use MCP tools for code generation and review. Always use the command-line toolchain (`cargo build`, `cargo test`, `cargo clippy`) to validate and compile code before committing changes. Prefer native compilation for Linux.

---

## 1. Overview

A desktop weather application built in **Rust** with a rich, modern UI powered by **Slint** and its **Material Components** ecosystem. The app fetches real-time weather data from public APIs, displays it in a tabbed interface, and provides a clean, native-compiled experience on Linux.

---

## 2. Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust (edition 2021 or 2024) |
| UI Framework | [Slint](https://slint.dev/) with Material Components (`slint-material-preview` or equivalent) |
| HTTP Client | `reqwest` (async, with TLS) |
| JSON Parsing | `serde` + `serde_json` |
| Async Runtime | `tokio` |
| Logging | `tracing` + `tracing-subscriber` (file + console) |
| Config / Settings | `serde` + `toml` (config file stored in `~/.config/AetherCast/`) |
| Native Compilation | `cargo build --release` → static binary (target: `x86_64-unknown-linux-gnu`) |

---

## 3. Architecture

```
┌─────────────────────────────────────────────┐
│                 UI Layer                     │
│  ┌──────────┬──────────┬────────┬─────────┐ │
│  │  Today   │ Tomorrow │  Week  │  About  │ │
│  └──────────┴──────────┴────────┴─────────┘ │
│         Slint .slint files + Material       │
├─────────────────────────────────────────────┤
│              Business Layer                  │
│  ┌───────────────────────────────────────┐  │
│  │         WeatherService (async)        │  │
│  │  • Fetch from API (multi-provider)    │
│  │  • Parse & normalise models           │
│  │  • Cache / retry / fallback           │
│  └───────────────────────────────────────┘  │
├─────────────────────────────────────────────┤
│             Data Layer                       │
│  ┌──────────┬──────────┬──────────────────┐ │
│  │  Open-Meteo│  wttr.in │  OpenWeatherMap │ │
│  │  (free)   │ (free)   │  (free tier)    │ │
│  └──────────┴──────────┴──────────────────┘ │
│  ┌──────────┬──────────┬──────────────────┐ │
│  │  WeatherAPI│  Visual  │  fallback mock  │ │
│  │  .com     │  Climate │  (dev / offline)│ │
│  └──────────┴──────────┴──────────────────┘ │
├─────────────────────────────────────────────┤
│           Infrastructure                     │
│  tracing (logs → ~/.local/share/AetherCast/)│
│  serde/toml (config)                         │
│  reqwest + tokio (HTTP)                      │
└─────────────────────────────────────────────┘
```

---

## 4. Data Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub date: NaiveDate,
    pub temperature: Temperature,
    pub pressure: f32,          // hPa
    pub wind_speed: f32,        // km/h
    pub wind_direction: String, // "N", "NE", "E", …
    pub humidity: u8,           // %
    pub condition: WeatherCondition,
    pub description: String,
    pub icon_code: String,      // maps to Slint icon / SVG
    pub feels_like: f32,
    pub uv_index: Option<f32>,
    pub precipitation: f32,     // mm
    pub visibility: Option<f32>, // km
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Temperature {
    pub current: f32,
    pub min: f32,
    pub max: f32,
    pub unit: TemperatureUnit, // Celsius or Fahrenheit
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeatherCondition {
    ClearSky,
    MainlyClear,
    PartlyCloudy,
    Overcast,
    Fog,
    Drizzle,
    Rain,
    HeavyRain,
    Snow,
    Thunderstorm,
    Unknown(String),
}
```

---

## 5. Tab Specifications

### 5.1 Today

| Element | Description |
|---|---|
| **Weather Icon** | Large SVG / Slint icon: ☀️ sun, ⛅ partly-cloudy, 🌧️ rain, ❄️ snow, ⛈️ thunderstorm, 🌫️ fog |
| **Temperature** | Current temp in large font, with min/max badges |
| **Feels Like** | Smaller label |
| **Pressure** | Value in hPa (e.g. `1013 hPa`) |
| **Wind** | Speed in km/h + compass arrow showing direction |
| **Humidity** | Percentage with a bar indicator |
| **Description** | Free-text (e.g. "Light rain and breeze") |
| **Precipitation** | Amount in mm |
| **UV Index** | Optional bar / colour-coded |
| **Visibility** | Optional in km |

### 5.2 Tomorrow

Same layout as **Today**, but data is for the next calendar day.

### 5.3 Week

A horizontal / vertical scrollable list (7 days, starting from tomorrow):

| Element per card | Description |
|---|---|
| **Day name** | "Mon", "Tue", … |
| **Weather icon** | Small (scaled-down version of Today icons) |
| **Min / Max temp** | Two small labels |
| **Condition** | One-line description |
| **Precipitation** | Small mm value |

### 5.4 About

| Element | Content |
|---|---|
| App name & version | e.g. "AetherCast v0.1.0" |
| Technologies used | Rust, Slint, reqwest, tokio, tracing, etc. |
| Weather APIs | List of providers, license / attribution |
| Credits | Slint contributors, Material Design icons |
| Open-source note | Link to repository |

---

## 6. Weather API Providers (with fallback chain)

The app should support **multiple providers** and attempt them in order. If one fails, log a warning and move to the next.

| # | Provider | Auth Required | Notes |
|---|---|---|---|
| 1 | **Open-Meteo** | ❌ No | Fully free, no API key. Best fallback. |
| 2 | **wttr.in** | ❌ No | Simple text / JSON API. Great for quick lookups. |
| 3 | **Open-Meteo (secondary)** | ❌ No | Same provider, different endpoint or parameters. |
| 4 | **OpenWeatherMap** | ✅ Free key | Very popular, generous free tier. |
| 5 | **WeatherAPI.com** | ✅ Free key | Good free tier, 1M requests/month. |
| 6 | **Visual Crossing** | ✅ Free key | 1000 requests/day free. |
| 7 | **Mock / Cached** | ❌ No | Offline fallback for development. |

### Provider selection

- Configurable in `config.toml` via `provider = "open_meteo"` (first choice).
- On startup, the app logs which provider is active.
- On error, each failed attempt is logged at `WARN` level and the next provider is tried.
- The last-resort fallback is a **mock provider** that returns deterministic sample data (useful for testing UI without network).

---

## 7. Configuration

File: `~/.config/AetherCast/config.toml`

```toml
[app]
name = "AetherCast"
version = "0.1.0"
theme = "material"  # "material", "dark", "light"

[location]
latitude = 52.2297
longitude = 21.0122
city = "Warsaw"

[weather]
provider = "open_meteo"
temperature_unit = "celsius"   # "celsius" or "fahrenheit"
wind_unit = "kmh"              # "kmh", "ms", "mph"
language = "en"

[logging]
level = "info"                 # "debug", "info", "warn", "error"
file = "~/.local/share/AetherCast/app.log"

[ui]
refresh_interval_seconds = 900  # 15 minutes
```

---

## 8. Logging Strategy

Use `tracing` throughout the codebase:

```rust
use tracing::{info, warn, error, debug, instrument};
```

| Level | When to use |
|---|---|
| `debug` | Request / response payloads, internal state transitions |
| `info` | App start, provider selection, successful fetch, config loaded |
| `warn` | Provider timeout, fallback triggered, missing optional field |
| `error` | Unrecoverable error, UI crash, config parse failure |

**Log destinations:**

1. **Console** — always active.
2. **File** — `~/.local/share/AetherCast/app.log` (rotated, max 5 MB, keep 5 files).
3. **Slint debug overlay** — optional toggle in About tab to show recent log lines.

---

## 9. Slint UI Structure

```
src/
├── main.rs                  # Entry point, app init, Slint runtime
├── app.slint                # Main window with tab bar
├── tabs/
│   ├── today.slint          # Today tab UI
│   ├── tomorrow.slint       # Tomorrow tab UI
│   ├── week.slint           # Week tab UI
│   └── about.slint          # About tab UI
├── icons/
│   ├── sun.slint            # Weather icon definitions
│   ├── cloud.slint
│   ├── rain.slint
│   ├── snow.slint
│   ├── thunderstorm.slint
│   ├── fog.slint
│   └── wind_compass.slint   # Wind direction compass
├── services/
│   ├── mod.rs
│   ├── weather_service.rs   # Provider abstraction + fetch logic
│   ├── open_meteo.rs        # Open-Meteo adapter
│   ├── wttr_in.rs           # wttr.in adapter
│   ├── open_weather_map.rs  # OpenWeatherMap adapter
│   ├── weather_api.rs       # WeatherAPI.com adapter
│   ├── visual_crossing.rs   # Visual Crossing adapter
│   └── mock.rs              # Offline mock data provider
├── models/
│   ├── mod.rs
│   └── weather.rs           # Shared data structures
├── config/
│   ├── mod.rs
│   └── app_config.rs        # Config loading / saving
└── logging/
    ├── mod.rs
    └── setup.rs             # tracing-subscriber initialisation
```

---

## 10. Implementation Phases

### Phase 1 — Project Setup & Build Pipeline

- [ ] `cargo init` with dependencies: `slint`, `reqwest`, `tokio`, `serde`, `serde_json`, `tracing`, `tracing-subscriber`, `chrono`, `toml`, `dirs`
- [ ] Configure `Cargo.toml` with `[package]`, `[dependencies]`, `[dev-dependencies]`
- [ ] Set up `tracing` subscriber (file + console)
- [ ] Create minimal `main.rs` that opens an empty Slint window
- [ ] **Validate:** `cargo build` and `cargo clippy` pass clean

### Phase 2 — Config & Logging

- [ ] Implement `config/app_config.rs` — load / save `config.toml`
- [ ] Default config with sensible values
- [ ] **Validate:** Unit tests for config parsing; `cargo test` passes

### Phase 3 — Data Models & Weather Service Abstraction

- [ ] Define `WeatherData`, `Temperature`, `WeatherCondition` structs
- [ ] Create `WeatherService` trait with `async fn fetch(&self, lat: f64, lon: f64) -> Result<WeatherData, Error>`
- [ ] Implement mock provider (returns hardcoded data)
- [ ] **Validate:** `cargo test` — mock provider tests

### Phase 4 — API Adapters

- [ ] Implement **Open-Meteo** adapter (no auth, primary)
- [ ] Implement **wttr.in** adapter
- [ ] Implement **OpenWeatherMap** adapter
- [ ] Implement **WeatherAPI.com** adapter
- [ ] Implement **Visual Crossing** adapter
- [ ] Wire the fallback chain: try providers in order, log each attempt
- [ ] **Validate:** Integration tests with real API calls (skip with `#[ignore]` unless `--include-ignored`); `cargo clippy` clean

### Phase 5 — Slint UI: Tab Bar & Navigation

- [ ] Create `app.slint` with tab bar (Today, Tomorrow, Week, About)
- [ ] Implement tab switching logic in Rust
- [ ] **Validate:** `cargo build` — window opens with working tab navigation

### Phase 6 — Slint UI: Today & Tomorrow Tabs

- [ ] `today.slint` — large icon, temperature, pressure, wind (arrow + speed), humidity bar, description
- [ ] `tomorrow.slint` — same layout, bound to tomorrow's data
- [ ] Bind Rust data to Slint properties via `set_weather_data()`
- [ ] Weather icons: define SVG-based icons in Slint
- [ ] **Validate:** `cargo build` — data renders correctly

### Phase 7 — Slint UI: Week Tab

- [ ] `week.slint` — scrollable list of 7 day cards
- [ ] Each card: day name, small icon, min/max temp, precipitation
- [ |  Pass weekly forecast data from Rust to Slint list model
- [ ] **Validate:** `cargo build` — scrollable week view renders

### Phase 8 — Slint UI: About Tab

- [ ] `about.slint` — app info, tech stack, API credits, version
- [ ] Optional: show recent log lines (debug feature)
- [ ] **Validate:** `cargo build` — About tab displays

### Phase 9 — Polish & Production

- [ ] Auto-refresh data every N seconds (configurable)
- [ ] Error handling UI (toast / banner when fetch fails)
- [ ] Loading spinner during fetch
- [ ] Dark / light theme toggle
- [ ] Proper window sizing & responsive layout
- [ ] Add `.desktop` file for Linux desktop integration
- [ ] Add `LICENSE`, `README.md`
- [ ] **Validate:** `cargo build --release` — final native binary

---

## 11. Error Handling & Resilience

| Scenario | Behaviour |
|---|---|
| All providers down | Show cached / mock data + error toast |
| Network timeout | Retry once with exponential back-off (1s → 2s → 4s) |
| Invalid API response | Log error, try next provider |
| Config file missing | Use built-in defaults, create file on exit |
| Slint UI panic | Catch and show error dialog; log stack trace |

---

## 12. Native Compilation (Linux)

```bash
# Debug build
cargo build

# Release build (optimised, static)
cargo build --release

# Output binary
./target/release/AetherCast
```

**Cross-compilation** (optional):
```bash
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu
```

**System dependencies** (may be needed):
```bash
# Ubuntu / Debian
sudo apt install libslint-dev libssl-dev pkg-config

# Fedora
sudo dnf install slint-devel openssl-devel pkg-config
```

---

## 13. CI / Validation Checklist (Manual or Automated)

- [ ] `cargo build` — compiles without errors
- [ ] `cargo clippy -- -D warnings` — no clippy warnings
- [ ] `cargo test` — all unit tests pass
- [ ] `cargo build --release` — release binary produces
- [ ] Manual test: app opens, tabs switch, data displays
- [ ] Manual test: all 7 weather icons render
- [ ] Manual test: config file created on first run
- [ ] Manual test: provider fallback works (disable primary provider)
- [ ] Log file created at configured path
- [ ] Binary runs on target Linux distro without extra deps

---

## 14. Future Enhancements (out of scope for v1)

- Geolocation (auto-detect city)
- Weather alerts / notifications
- Historical data & charts
- Multiple city profiles
- Widget / system tray icon
- Mobile port (Slint supports Android / iOS)

---

## 15. Design Notes for AI Agent

1. **Always validate code with the command line.** After every code change, run `cargo build` and `cargo clippy` to ensure the code compiles and passes linting. Do not assume correctness without building.

2. **Prefer MCP tools** for code review, dependency analysis, and documentation lookups. Use them to verify API schemas and Slint property bindings before writing code.

3. **Slint Material Components** — use the Material Design-inspired widgets provided by Slint (buttons, cards, tabs, sliders, progress bars). Keep colours consistent with Material Design palette.

4. **Icons** — use Slint's built-in icon support or inline SVGs. Create a mapping from `WeatherCondition` → icon file name.

5. **Logging first, debugging later.** Every async fetch, config load, and UI action should be logged. This makes post-release bug fixing much easier.

6. **Graceful degradation.** The app must never crash if a provider is unavailable. Always have a fallback path.
