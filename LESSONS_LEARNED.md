# Lessons Learned — Rust + Slint AetherCast

## Overview

This document records every major problem encountered while building a desktop weather application using **Rust** and **Slint 1.18**, along with the solutions found and best practices to prevent these issues in future projects.

---

## 1. Slint Syntax: The Biggest Challenge

Slint has a unique declarative syntax that differs significantly from CSS, XAML, QML, or any other UI markup language. The compiler is strict and provides cryptic error messages.

### 1.1 Property Declaration Syntax

**Problem:** Used `property<WeatherData> data;` (C++-style) instead of Slint's actual syntax.

**Solution:** Slint uses `property<type> name;` for internal properties and `in property<type> name;` for externally-assigned properties.

```slint
// Internal (read-write within component)
property<int> active-tab: 0;

// External assignment (set from parent)
in property<WeatherData> data;

// Bidirectional binding
in-out property<bool> loading: false;
```

**Best Practice:** Always use `in property` when a parent component needs to set a property. Use `in-out` when the child component needs to modify the property and have changes reflected back.

### 1.2 Struct vs Global

**Problem:** Defined `WeatherData` as `export global WeatherData { ... }` (a singleton), then tried to use it as a property type. Globals are singletons — you can't have multiple instances.

**Solution:** Use `export struct WeatherData { ... }` for data types that need multiple instances.

```slint
// Wrong — global is a singleton
export global WeatherData {
    in-out property<string> date;
    // ...
}

// Correct — struct is a type, not a singleton
export struct WeatherData {
    date: string,
    temperature: string,
    // ...
}
```

**Best Practice:** Use `struct` for data models, `global` only for shared singletons (like a logging service or app context). Use `enum` for enumerations.

### 1.3 Struct Field Syntax

**Problem:** Used `property<string> date;` inside a struct definition.

**Solution:** Struct fields use `name: type,` syntax (no `property` keyword).

```slint
// Wrong
export struct WeatherData {
    property<string> date;
}

// Correct
export struct WeatherData {
    date: string,
    temperature: string,
}
```

**Best Practice:** Remember: components use `property`, structs use `name: type`.

### 1.4 Layout Elements

**Problem:** Used `VerticalLayout` and `HorizontalLayout` (non-existent).

**Solution:** Slint uses `VerticalLayout` and `HorizontalLayout` as **built-in elements** (not from std-widgets). The std-widgets have `VerticalBox` and `HorizontalBox` which are different.

**Best Practice:** Use `VerticalLayout` / `HorizontalLayout` for layout. They are built-in and always available.

### 1.5 `for` Loop Syntax

**Problem:** Used `for i in 0..<Tab.count` and `for day in week-data { }` with curly braces.

**Solution:** Slint `for` loops use a colon `:` after the expression, and the body is a single element.

```slint
// Wrong
for i in 0..<Tab.count {
    TabButton { ... }
}

// Correct
for day in week-data :
    WeekDayCard {
        day: day;
    }
```

**Best Practice:** Always use `:` after `for` expressions. The body is always a single element (not a block).

### 1.6 Conditional Element Syntax

**Problem:** Used `if condition { element }` with curly braces.

**Solution:** Slint conditionals use `if condition :` with a colon, and the body is a single element.

```slint
// Wrong
if subtitle != "" {
    Text { text: subtitle; }
}

// Correct
if subtitle != "" :
    Text { text: subtitle; }
```

**Best Practice:** Remember `if condition :` (colon, not braces).

### 1.7 Ternary Expressions in Property Bindings

**Problem:** Used `font-weight: Bold` (not a valid identifier) and `font-weight: is-active ? Bold : Regular` (ternary with enum values).

**Solution:** Slint ternary uses `? :` syntax, but font-weight uses `FontWeight.bold` (lowercase, dot notation).

```slint
// Wrong
font-weight: Bold;
font-weight: is-active ? Bold : Regular;

// Correct
font-weight: FontWeight.bold;
font-weight: is-active ? FontWeight.bold : FontWeight.normal;
```

**Best Practice:** Slint font weights: `FontWeight.bold`, `FontWeight.normal`, `FontWeight.medium`, `FontWeight.semi-bold` (all lowercase with hyphens).

### 1.8 Alignment Values

**Problem:** Used `alignment: center` and `horizontal-alignment: center`.

**Solution:** Use `LayoutAlignment.center` for layout alignment and `center` (lowercase) for text alignment.

```slint
// Wrong
alignment: center;

// Correct
alignment: LayoutAlignment.center;
horizontal-alignment: center;
```

**Best Practice:** Layout alignment uses the `LayoutAlignment` namespace. Text alignment uses simple lowercase values.

### 1.9 Array Types

**Problem:** Used `<WeatherData>` (angle brackets) for array types.

**Solution:** Slint arrays use square brackets `[WeatherData]`.

```slint
// Wrong
property<WeatherData> week-data;
callback set_week_weather(<WeatherData>);

// Correct
property<[WeatherData]> week-data;
callback set_week_weather([WeatherData]);
```

**Best Practice:** Arrays always use `[Type]` syntax in Slint.

### 1.10 Border Properties

**Problem:** Used `border-width: 0 0 2px 0` (CSS shorthand) and `border-bottom: 3px solid color`.

**Solution:** Slint doesn't support CSS shorthand. Use individual properties.

```slint
// Wrong
border-width: 0 0 2px 0;
border-bottom: 3px solid rgb(33, 150, 243);

// Correct
border-bottom-width: 2px;
border-bottom-color: rgb(232, 234, 236);
```

**Best Practice:** Slint uses `border-{side}-width` and `border-{side}-color` separately.

### 1.11 ScrollView vs ListView

**Problem:** Used `ScrollView` with `ScrollOrientation: Vertical` (non-existent property) and tried to put a `for` loop inside `ScrollView`.

**Solution:** `ScrollView` doesn't have a `ScrollOrientation` property. For lists, use `ListView` which automatically handles scrolling and `for` loops.

```slint
// Wrong — ScrollView doesn't support for loops directly
ScrollView {
    ScrollOrientation: Vertical;
    for day in week-data :
        WeekDayCard { ... }
}

// Correct — use ListView for lists
ListView {
    for day in week-data :
        WeekDayCard { ... }
}
```

**Best Practice:** Use `ListView` for scrollable lists with `for` loops. Use `ScrollView` only for general scrollable content.

### 1.12 TouchArea Placement

**Problem:** Put `TouchArea` as a direct child with its own `width`/`height`/`background`.

**Solution:** `TouchArea` is a built-in element that should be nested inside a container that provides the visual appearance.

```slint
// Wrong — TouchArea can't have its own size in this context
TouchArea {
    width: 36px;
    height: 36px;
    background: white;
    clicked => { ... }
}

// Correct — TouchArea inside a Rectangle
Rectangle {
    width: 36px;
    height: 36px;
    background: white;
    TouchArea {
        clicked => { ... }
    }
}
```

**Best Practice:** `TouchArea` handles input; a parent `Rectangle` handles visuals.

### 1.13 Padding on Non-Layout Elements

**Problem:** Used `padding: 16px` on `Rectangle` elements.

**Solution:** `padding` only works on layout elements. On `Rectangle`, use child elements with positioning.

**Best Practice:** Only use `padding` on `VerticalLayout`, `HorizontalLayout`, and other layout containers.

### 1.14 Component Inheritance

**Problem:** Used `extends Window` instead of `inherits Window`.

**Solution:** Slint uses `inherits` for component inheritance.

```slint
// Wrong
export component AppWindow extends Window { }

// Correct
export component AppWindow inherits Window { }
```

**Best Practice:** Always use `inherits`, never `extends`.

### 1.15 Private Properties

**Problem:** Tried to assign to a property defined without `in` from a parent component.

**Solution:** Properties without `in` are private to the component. Use `in property` for external assignment.

```slint
// Component definition
export component WeatherIcon inherits Rectangle {
    in property<string> icon-code;  // Can be set from outside
}

// Parent usage
WeatherIcon {
    icon-code: data.icon-code;  // Works because property is 'in'
}
```

**Best Practice:** Always declare externally-settable properties with `in property`.

---

## 2. Rust + Slint Integration

### 2.1 Type Name Collisions

**Problem:** Imported `models::weather::*` and then used `WeatherData` — but `WeatherData` is also a Slint-generated type. The Rust compiler couldn't distinguish them.

**Solution:** Use the `weather` module prefix for Rust types.

```rust
// Wrong
use models::weather::*;
fn to_slint_weather(data: &WeatherData) -> WeatherData { ... }
// ^^^^^^^^^^^^ ambiguous: is this Rust or Slint?

// Correct
use models::weather;
fn to_slint_weather(data: &weather::WeatherData) -> WeatherData { ... }
// ^^^^^^^^^^^^^^^^^ Rust type      ^^^^^^^^^^^^ Slint type
```

**Best Practice:** Import modules, not glob patterns, when Slint-generated types share names with Rust types.

### 2.2 Slint Struct Field Names

**Problem:** Slint struct fields with hyphens (`temp-min`) get generated as snake_case in Rust (`temp_min`).

**Solution:** Use the snake_case version in Rust code.

```rust
// Slint struct
export struct WeatherData {
    temp-min: string,
    temp-max: string,
}

// Rust accessor
WeatherData {
    temp_min: "...".into(),   // snake_case in Rust
    temp_max: "...".into(),
}
```

**Best Practice:** Slint converts hyphenated names to snake_case in Rust. Verify generated names with `cargo expand` or the generated code.

### 2.3 SharedString vs String

**Problem:** Tried to use `.unwrap_or()` on `slint::SharedString` (which is not `Option<T>`).

**Solution:** `SharedString` is a plain string type, not an `Option`. Use conditional logic.

```rust
// Wrong — SharedString is not Option
w.set_uv_index(data.uv_index.unwrap_or(0.0));

// Correct
w.set_uv_index(if let Some(uv) = data.uv_index {
    format!("{:.1}", uv).into()
} else {
    "".into()
});
```

**Best Practice:** Slint `SharedString` is always present (never `None`). Convert Rust `Option` to `SharedString` explicitly.

### 2.4 ModelRc Conversion

**Problem:** Tried to convert `VecModel` directly to `ModelRc`.

**Solution:** Wrap in `Rc` first, then convert.

```rust
fn to_slint_week(week: &[weather::WeatherData]) -> slint::ModelRc<WeatherData> {
    let vec: Vec<WeatherData> = week.iter().map(to_slint_weather).collect();
    slint::ModelRc::from(std::rc::Rc::new(slint::VecModel::from(vec)))
}
```

**Best Practice:** `ModelRc::from(Rc<VecModel<T>>)` is the correct conversion path.

### 2.5 Component Cloning

**Problem:** Used `.clone()` on Slint components.

**Solution:** Slint components use `.clone_strong()` for strong references.

```rust
// Wrong
let app_clone = app.clone();

// Correct
let app_clone = app.clone_strong();
```

**Best Practice:** Always use `clone_strong()` for Slint component handles.

### 2.6 Callback Return Types

**Problem:** Used `.ok()` on callback setters that return `()`.

**Solution:** Callback setters return `()`, not `Result`.

```rust
// Wrong
app.set_loading(true)?;
app.set_error("".to_string()).ok();

// Correct
app.set_loading(true);
app.set_error_message("".into());
```

**Best Practice:** Slint callback setters return `()`. Don't use `?` or `.ok()` on them.

### 2.7 Thread Safety

**Problem:** Tried to use `tokio::spawn` with Slint component handles. Slint components are not `Send` + `Sync`.

**Solution:** Run fetch synchronously on the event loop thread, or use `std::thread::spawn` with a new Tokio runtime per thread.

```rust
// Wrong — Slint components aren't Send
tokio::spawn(async move {
    run_fetch(&app).await;
});

// Correct — synchronous fetch on event loop thread
app.on_refresh_weather(move || {
    let config = load_config();
    match fetch_weather_sync(&config) {
        Ok(data) => { app.set_today_weather(...); }
        Err(e) => { app.set_error_message(...); }
    }
});
```

**Best Practice:** Keep Slint component operations on the event loop thread. For background work, either run synchronously or spawn a thread with its own runtime.

---

## 3. Build System & Dependencies

### 3.1 slint-build vs slint! Macro

**Problem:** Unclear whether to use `slint_build::compile()` in `build.rs` or the `slint!` macro in Rust.

**Solution:** Use `slint_build::compile()` in `build.rs` for file-based UI, then `slint::include_modules!()` in Rust.

```rust
// build.rs
fn main() {
    slint_build::compile("src/app.slint").unwrap();
}

// main.rs
slint::include_modules!();
```

**Best Practice:** File-based UI → `build.rs` + `include_modules!()`. Inline UI → `slint!` macro.

### 3.2 Tracing-Subscriber Features

**Problem:** Used `tracing_subscriber::registry().with(...)` but the `registry` feature wasn't enabled.

**Solution:** Enable the `registry` feature in `Cargo.toml`.

```toml
tracing-subscriber = { version = "0.3", features = ["env-filter", "json", "fmt", "registry"] }
```

**Best Practice:** Always check which features are needed for the APIs you use. The `registry` feature is required for `.with()` chaining.

### 3.3 Default Derives

**Problem:** Structs used in `load_config()` required `Default` but didn't derive it.

**Solution:** Add `Default` derive to all config sub-structs.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfigSection { ... }
```

**Best Practice:** When using `serde` deserialization with fallback defaults, derive `Default` on all config structs.

---

## 4. Weather Service Implementation

### 4.1 Static Str vs String

**Problem:** `condition_to_icon()` returns `&'static str`, but `WeatherData.icon_code` is `String`.

**Solution:** Call `.to_string()` on static strings when assigning to `String` fields.

```rust
// Wrong
icon_code: condition_to_icon(&condition),  // &'static str

// Correct
icon_code: condition_to_icon(&condition).to_string(),
```

**Best Practice:** Always check type compatibility between `&str` and `String` in Rust.

### 4.2 Thread-Safe RNG

**Problem:** `rand::rngs::ThreadRng` is not `Send` + `Sync`, breaking `async_trait` impl.

**Solution:** Use `rand::random!()` or `fastrand` for thread-safe random generation.

```rust
// Wrong — ThreadRng is not Send
pub struct MockProvider {
    rng: rand::rngs::ThreadRng,
}

// Correct — no state needed
pub struct MockProvider;
```

**Best Practice:** Async traits require `Send + Sync`. Avoid non-thread-safe state in async providers.

---

## 5. General Best Practices

### 5.1 Start Simple, Then Iterate

**Lesson:** The biggest time sink was trying to write the full UI at once and then debugging 50+ errors. 

**Best Practice:**
1. Start with a minimal Slint file that compiles
2. Add one component at a time
3. Build after each change to catch errors early
4. Verify each feature works before moving on

### 5.2 Read the Slint Documentation

**Lesson:** Slint's docs are thorough but hard to navigate. Many errors came from guessing syntax instead of looking it up.

**Best Practice:**
- Bookmark the Slint Language Reference: https://docs.slint.dev/latest/docs/slint/reference/language/
- Check the std-widgets reference for available widgets
- Look at Slint examples on GitHub: https://github.com/slint-ui/slint/tree/master/examples

### 5.3 Use `cargo expand` for Generated Code

**Lesson:** Not knowing what Slint generates for your `.slint` files made debugging type mismatches impossible.

**Best Practice:**
```bash
cargo expand --bin AetherCast 2>/dev/null | grep -A20 "pub struct WeatherData"
```

### 5.4 Single-File Approach for Small Projects

**Lesson:** Splitting UI across multiple `.slint` files with cross-imports caused circular dependency issues.

**Best Practice:** For small projects, keep the entire UI in one `app.slint` file. Split only when the file becomes unmanageable (>500 lines).

### 5.5 Test the Slint Compilation Early

**Lesson:** The Slint compiler runs as a build script step, so Rust errors are hidden behind Slint errors.

**Best Practice:**
1. Fix Slint compilation errors first
2. Then address Rust compilation errors
3. Use `cargo clean` between major Slint changes to avoid stale generated code

### 5.6 Error Message Order

**Lesson:** The first Slint error often cascades into dozens of downstream errors.

**Best Practice:** Fix errors top-to-bottom in the file. The first error is usually the root cause.

### 5.7 Use `#[allow(clippy::...)]` for External API Fields

**Lesson:** Clippy complains about CamelCase field names from API responses.

**Best Practice:**
```rust
#[allow(clippy::upper_case_acronyms)]
struct WttrCurrent {
    temp_c: String,
    windspeedKmph: String,
    feelslikeC: String,
}
```

---

## 6. Layout Gotchas (2026)

### 6.1 ScrollView Inner Layout Must Not Have `vertical-stretch`

**Problem:** Added `vertical-stretch: 1` to the inner `VerticalLayout` of a `ScrollView`. The ScrollView appeared but content below the header was cut off — no scrolling.

**Solution:** `vertical-stretch: 1` on the inner layout pins it to the ScrollView's viewport size, preventing it from growing. Remove it so the content can grow naturally beyond the viewport.

```slint
// Wrong — inner layout pinned to viewport, no scroll
ScrollView {
    vertical-stretch: 1;
    VerticalLayout {
        vertical-stretch: 1;  // ← pins content, kills scroll
        // ... all content here ...
    }
}

// Correct — content grows naturally, ScrollView scrolls
ScrollView {
    vertical-stretch: 1;
    VerticalLayout {
        // no vertical-stretch here
        // ... all content ...
    }
}
```

**Best Practice:** `ScrollView` needs its inner content to be *taller* than the viewport to scroll. `vertical-stretch` on the inner layout prevents that growth.

### 6.2 `preferred-height: 0` Collapses ScrollView Entirely

**Problem:** Set `preferred-height: 0` on a `ScrollView` or its parent component. The entire scrollable area disappeared — nothing rendered below the header.

**Solution:** `preferred-height: 0` overrides `vertical-stretch: 1`. Remove `preferred-height: 0` from both the `ScrollView` and its parent component. Let `vertical-stretch: 1` alone handle sizing.

```slint
// Wrong — preferred-height: 0 collapses everything
export component AboutTab inherits Rectangle {
    preferred-height: 0;  // ← collapses the component
    vertical-stretch: 1;

    ScrollView {
        preferred-height: 0;  // ← collapses the scroll area
        vertical-stretch: 1;
        // ... content ...
    }
}

// Correct — vertical-stretch alone fills the space
export component AboutTab inherits Rectangle {
    vertical-stretch: 1;  // no preferred-height: 0

    ScrollView {
        vertical-stretch: 1;  // no preferred-height: 0
        // ... content ...
    }
}
```

**Best Practice:** Use `vertical-stretch: 1` for filling available space. Never combine it with `preferred-height: 0` — the zero wins and collapses everything.

### 6.3 `visible: false` Still Consumes Layout Space

**Problem:** Used `visible:` bindings on tab content elements to show/hide tabs. The visible tab was squeezed to a tiny area because invisible elements still reserved layout space.

**Solution:** Use Slint's `match` element for tab switching. Only the matched branch renders; non-matched branches take zero layout space.

```slint
// Wrong — visible: false still reserves layout space
VerticalLayout {
    TodayTab { visible: active-tab == 0; }
    WeekTab { visible: active-tab == 1; }
    AboutTab { visible: active-tab == 2; }
}

// Correct — match element takes zero space for non-matched branches
VerticalLayout {
    vertical-stretch: 1;
    match active-tab {
        0: TodayTab { data: today-weather; }
        1: WeekTab { week-data: week-weather; }
        2: AboutTab { }
        *: { }
    }
}
```

**Best Practice:** Use `match` for tab switching. Never use `visible:` for layout-dependent show/hide.

### 6.4 `match` Is an Experimental Slint Feature

**Problem:** Used `match active-tab { ... }` in a `.slint` file. Build failed with "Syntax error: expected Identifier".

**Solution:** `match` is experimental in Slint. Set `SLINT_ENABLE_EXPERIMENTAL_FEATURES=1` in `build.rs` before calling `slint_build::compile()`.

```rust
// build.rs
fn main() {
    unsafe { std::env::set_var("SLINT_ENABLE_EXPERIMENTAL_FEATURES", "1"); }
    slint_build::compile("src/app.slint").unwrap();
}
```

**Best Practice:** Any `.slint` file using `match` requires the experimental feature flag. Without it, compilation fails silently behind the Slint compiler error.

### 6.5 Rust Edition 2024: `std::env::set_var` Requires `unsafe`

**Problem:** Upgraded to Rust edition 2024. `build.rs` compiled fine under 2021 but failed under 2024 with `std::env::set_var` flagged as unsafe.

**Solution:** Wrap `std::env::set_var` calls in `unsafe {}` blocks in `build.rs`.

```rust
// Rust 2021 — worked without unsafe
std::env::set_var("SLINT_ENABLE_EXPERIMENTAL_FEATURES", "1");

// Rust 2024 — requires unsafe
unsafe { std::env::set_var("SLINT_ENABLE_EXPERIMENTAL_FEATURES", "1"); }
```

**Best Practice:** When upgrading to Rust 2024, check `build.rs` for `std::env::set_var` and wrap in `unsafe {}`.

---

## 7. Quick Reference Cheat Sheet

| Concept | Wrong | Correct |
|---------|-------|---------|
| Component inheritance | `extends Window` | `inherits Window` |
| Property declaration | `property<type> name;` | `in property<type> name;` (external) |
| Struct field | `property<string> name;` | `name: string,` |
| For loop | `for x in items { }` | `for x in items :` |
| If element | `if cond { }` | `if cond :` |
| Array type | `<WeatherData>` | `[WeatherData]` |
| Font weight | `Bold` | `FontWeight.bold` |
| Layout alignment | `center` | `LayoutAlignment.center` |
| Text alignment | `LayoutAlignment.center` | `center` |
| Callback setter | `app.set_foo(x)?` | `app.set_foo(x);` |
| Clone | `app.clone()` | `app.clone_strong()` |
| Border shorthand | `border: 1px solid red` | `border-width: 1px; border-color: red;` |
| ScrollView for lists | `ScrollView { for ... }` | `ListView { for ... }` |
| Padding on Rectangle | `padding: 12px` | Use child elements |
| TouchArea | `TouchArea { width: ... }` | `Rectangle { TouchArea { } }` |
| Struct vs Global | `export global Foo { }` | `export struct Foo { }` |
| ScrollView inner layout | `VerticalLayout { vertical-stretch: 1; }` | No `vertical-stretch` on inner layout |
| ScrollView parent | `preferred-height: 0` | `vertical-stretch: 1` only |
| Tab switching | `visible: active-tab == N` | `match active-tab { N: ... }` |
| match element | plain `.slint` file | `SLINT_ENABLE_EXPERIMENTAL_FEATURES=1` in `build.rs` |

---

## 8. Resources

- **Slint Documentation**: https://docs.slint.dev/
- **Slint GitHub Examples**: https://github.com/slint-ui/slint/tree/master/examples
- **Slint Language Reference**: https://docs.slint.dev/latest/docs/slint/reference/language/
- **Slint std-widgets**: https://docs.slint.dev/latest/docs/slint/reference/std-widgets/
- **Slint Issues**: https://github.com/slint-ui/slint/issues
