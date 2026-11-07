# AetherCast — Agent Instructions

## Quick Start

```bash
cargo build          # compile (Slint compiles via build.rs)
cargo run            # run desktop app
```

Single binary, no workspace. Slint UI files live directly in `src/`.

## Architecture

```
src/
├── main.rs                 # Entry point, provider chain, Slint bindings
├── app.slint               # Main window: header, tab bar, match-based tab switching
├── today.slint             # Today tab (Rectangle → VerticalLayout)
├── tomorrow.slint          # Tomorrow tab (inherits TodayTab)
├── week.slint              # Week tab (Rectangle → VerticalLayout → ListView)
├── about.slint             # About tab (Rectangle → ScrollView → VerticalLayout)
├── shared.slint            # WeatherIcon, InfoCard, HumidityBar, TabButton, LoadingSpinner
├── weather-data.slint      # export struct WeatherData (shared type)
├── config/                 # TOML config loading (~/.config/AetherCast/)
├── logging/                # tracing-subscriber setup (logs → ~/.local/share/AetherCast/)
├── models/weather.rs       # Rust data models
└── services/               # 6 weather providers + mock (fallback chain in main.rs)
```

Tab switching uses `match active-tab` (experimental Slint feature). `build.rs` sets `SLINT_ENABLE_EXPERIMENTAL_FEATURES=1`.

## Slint Gotchas (High Cost When Missed)

- **`match` is experimental** — `build.rs` must set `SLINT_ENABLE_EXPERIMENTAL_FEATURES=1`, or compilation fails.
- **Never use `vertical-stretch: 1` on the inner `VerticalLayout` of a `ScrollView`** — it pins content to the viewport, removing scroll.
- **Never use `preferred-height: 0` on a `ScrollView` or its parent component** — collapses the entire scrollable area. Use `vertical-stretch: 1` alone.
- **`visible: false` still consumes layout space** — use `match` for tab content, not per-child `visible:` bindings.
- **`for` loops use `:` not `{}`** — `for day in week-data : Rectangle { ... }`
- **`if` elements use `:` not `{}`** — `if subtitle != "" : Text { text: subtitle; }`
- **Array types use `[Type]`** — not `<Type>`.
- **Component inheritance uses `inherits`** — never `extends`.
- **`clone_strong()`** on Slint component handles — not `.clone()`.
- **`in property`** for externally-settable properties; plain `property` is internal only.
- **`ListView` for `for` loops**, `ScrollView` for general scrollable content.
- **Struct fields with hyphens** (`temp-min`) become snake_case in Rust (`temp_min`).
- **Only layout elements accept `padding`** — not `Rectangle`.

## Rust + Slint Integration

- **Name collision**: Slint generates types like `WeatherData`. Rust types are in modules (`weather::WeatherData`). Never glob-import `models::weather::*`.
- **`SharedString` is never `Option`** — convert Rust `Option` explicitly.
- **`ModelRc::from(Rc<VecModel<T>>)`** is the correct conversion path for list data.
- **Callback setters return `()`** — no `?` or `.ok()`.
- **Slint components are not `Send`** — run fetch synchronously on the event loop thread, or spawn a thread with its own Tokio runtime.
- **`app.show()` must come before `slint::run_event_loop()`** in `main.rs`.

## Dependencies (Current Versions)

| Crate | Version | Notes |
|-------|---------|-------|
| slint | 1.18 | `backend-default`, `renderer-femtovg` features |
| slint-build | 1.18 | File-based compile in `build.rs` |
| reqwest | 0.13 | `rustls` feature (not `rustls-tls`) |
| toml | 1 | TOML 1.1 spec |
| dirs | 7 | |
| thiserror | 1 | Constraint "2" fails (transitive deps pull 1.x) |
| Rust edition | 2024 | `build.rs` uses `unsafe {}` for `std::env::set_var` |

## Provider Fallback Chain

`fetch_weather_sync()` in `main.rs` tries providers in config order, then adds Open-Meteo, wttr.in, and any configured API-key providers as fallbacks. Mock provider is always last. Each failure logs `WARN` and continues.

## What's Not Done

- No tests (only `tempfile` in dev-dependencies).
- No CI configuration.
- No `.desktop` file for Linux desktop integration.
