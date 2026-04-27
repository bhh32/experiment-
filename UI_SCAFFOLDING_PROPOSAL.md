# libcosmic UI Scaffolding Proposal

> **Status:** Draft. The Figma at `figma.com/design/z3cclDojPTBVezkCMAT0mk` could
> not be accessed from this environment (Figma requires authentication / a
> rendered client). The structure below is a conventional libcosmic application
> scaffold; once the Figma frames, screen names, and component inventory are
> shared, this doc will be revised to name concrete pages, widgets, and state.

## Goals

- Stand up a libcosmic application that compiles and runs as an empty shell.
- Establish the file layout, module boundaries, and message-routing pattern
  before any pixel work begins.
- Keep i18n, configuration, and theming wired in from the first commit so they
  do not need to be retrofitted later.
- Mirror conventions used by COSMIC first-party apps (`cosmic-files`,
  `cosmic-edit`, `cosmic-settings`) so contributors can move between codebases
  without re-learning the layout.

## Open questions (please confirm against Figma)

1. **Surface type** — Is this a windowed application (`cosmic::app::Application`)
   or a panel applet (`cosmic::applet`)? The two have different entry points
   and widget conventions.
2. **Screen inventory** — How many top-level views? Navigation pattern
   (sidebar, tabs, single page, wizard)?
3. **Persistence** — Are there user-editable preferences? If so, the `Config`
   module should be wired to `cosmic-config`.
4. **Async work** — Network calls, file I/O, subprocess? Drives whether we
   add `tokio` and `Subscription` plumbing.
5. **Localization scope** — Which languages should the i18n stubs cover at
   scaffold time? English only is fine for v0.

## Proposed file layout

```
.
├── Cargo.toml
├── README.md
├── LICENSE
├── justfile
├── i18n.toml
├── i18n/
│   └── en/
│       └── <app-id>.ftl
├── data/
│   ├── icons/
│   │   └── hicolor/scalable/apps/<app-id>.svg
│   ├── <app-id>.desktop
│   └── <app-id>.metainfo.xml
└── src/
    ├── main.rs
    ├── app.rs
    ├── config.rs
    ├── localize.rs
    ├── pages/
    │   ├── mod.rs
    │   └── home.rs
    └── widgets/
        └── mod.rs
```

---

## File-by-file proposal

### `Cargo.toml`

**Purpose:** Crate manifest. Pins libcosmic to the upstream git rev used by
COSMIC ecosystem apps and enables the feature set the UI needs.

**Contents (sketch):**

```toml
[package]
name = "<app-name>"
version = "0.1.0"
edition = "2024"

[dependencies]
i18n-embed       = { version = "0.16", features = ["fluent-system", "desktop-requester"] }
i18n-embed-fl    = "0.10"
rust-embed       = "8"
serde            = { version = "1", features = ["derive"] }
tokio            = { version = "1", features = ["full"] }   # only if async work is needed

[dependencies.libcosmic]
git              = "https://github.com/pop-os/libcosmic.git"
default-features = false
features         = ["wayland", "tokio", "desktop", "winit"] # adjust per surface

[features]
default = []
```

**Decisions to make:** applet vs. application features; whether to enable
`xdg-portal` / `rfd` for file dialogs.

---

### `src/main.rs`

**Purpose:** Process entry point. Initializes localization, then hands control
to libcosmic's runtime with the `App` from `app.rs`.

**Contents (sketch):**

```rust
mod app;
mod config;
mod localize;
mod pages;
mod widgets;

fn main() -> cosmic::iced::Result {
    localize::init();
    let settings = cosmic::app::Settings::default();
    cosmic::app::run::<app::App>(settings, ())
}
```

**Why a thin `main.rs`:** keeps the entry point boring so contributors look in
`app.rs` for behavior.

---

### `src/app.rs`

**Purpose:** Implements `cosmic::Application` (or `cosmic::applet::Applet`).
Owns the root `Message` enum, the application state struct, and the
`init` / `update` / `view` / `subscription` lifecycle.

**Sketch:**

```rust
use cosmic::{Application, Element, app::Core, executor};

pub struct App {
    core: Core,
    page: pages::Page,
    config: config::Config,
}

#[derive(Clone, Debug)]
pub enum Message {
    Page(pages::Message),
    ConfigChanged(config::Config),
}

impl Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "com.example.AppName";

    fn core(&self) -> &Core { &self.core }
    fn core_mut(&mut self) -> &mut Core { &mut self.core }

    fn init(core: Core, _: Self::Flags) -> (Self, cosmic::Task<Self::Message>) { /* ... */ }
    fn update(&mut self, msg: Self::Message) -> cosmic::Task<Self::Message> { /* ... */ }
    fn view(&self) -> Element<Self::Message> { self.page.view().map(Message::Page) }
}
```

**Decisions to make:** whether the app needs a `header_start` / `header_end` /
`nav_bar` (sidebar) — those become extra trait methods.

---

### `src/config.rs`

**Purpose:** Strongly-typed user preferences, persisted via `cosmic-config`
(versioned, atomic, watched). Even if there are no settings yet, scaffolding
this now means the first preference can land without a refactor.

**Sketch:**

```rust
use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};
use serde::{Deserialize, Serialize};

pub const CONFIG_VERSION: u64 = 1;

#[derive(Clone, Debug, Default, Serialize, Deserialize, CosmicConfigEntry, PartialEq, Eq)]
#[version = 1]
pub struct Config {
    // pub theme: Theme,
    // pub last_open_path: Option<String>,
}
```

---

### `src/localize.rs`

**Purpose:** Initializes the Fluent localizer that `fl!()` macros across the
app rely on. Embeds `i18n/` at compile time via `rust-embed`.

**Sketch:**

```rust
use i18n_embed::{
    DesktopLanguageRequester,
    fluent::{FluentLanguageLoader, fluent_language_loader},
};
use rust_embed::RustEmbed;
use std::sync::LazyLock;

#[derive(RustEmbed)] #[folder = "i18n/"]
struct Localizations;

pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader = fluent_language_loader!();
    loader.load_fallback_language(&Localizations).expect("fallback locale missing");
    loader
});

pub fn init() {
    let requested = DesktopLanguageRequester::requested_languages();
    let _ = i18n_embed::select(&*LANGUAGE_LOADER, &Localizations, &requested);
}

#[macro_export]
macro_rules! fl {
    ($id:literal) => {{ i18n_embed_fl::fl!($crate::localize::LANGUAGE_LOADER, $id) }};
    ($id:literal, $($k:tt = $v:expr),* $(,)?) => {{
        i18n_embed_fl::fl!($crate::localize::LANGUAGE_LOADER, $id, $($k = $v),*)
    }};
}
```

---

### `src/pages/mod.rs`

**Purpose:** Page registry. Re-exports each page module and defines a top-level
`Page` enum + `Message` enum so `app.rs` only knows about one type per
concern, regardless of how many pages exist.

**Sketch:**

```rust
pub mod home;

pub enum Page {
    Home(home::State),
}

#[derive(Clone, Debug)]
pub enum Message {
    Home(home::Message),
}

impl Page {
    pub fn view(&self) -> cosmic::Element<Message> { /* dispatch */ }
    pub fn update(&mut self, msg: Message) -> cosmic::Task<Message> { /* dispatch */ }
}
```

**Once Figma frames are mapped:** add one module per top-level screen
(`pages/library.rs`, `pages/details.rs`, etc.). Each module owns its own
`State` + `Message` and never reaches into another page's state.

---

### `src/pages/home.rs`

**Purpose:** Placeholder first page so the app launches to something visible.
Replaced once the Figma's landing screen is identified.

**Sketch:**

```rust
use cosmic::{Element, widget};

#[derive(Default)]
pub struct State;

#[derive(Clone, Debug)]
pub enum Message {}

pub fn view(_state: &State) -> Element<'_, Message> {
    widget::text::title1(crate::fl!("welcome")).into()
}
```

---

### `src/widgets/mod.rs`

**Purpose:** Project-local reusable widgets — composed cosmic primitives that
appear in more than one page (e.g. labelled rows, empty states, header bars
with consistent spacing). Empty at scaffold time; fill as duplication appears.

**Rule of thumb:** do not pre-create widgets here. Add a module the second
time the same composition shows up.

---

### `i18n.toml`

**Purpose:** Tells `cargo i18n` and `i18n-embed` which fallback language and
fluent assets to use.

**Contents:**

```toml
fallback_language = "en"
[fluent]
assets_dir = "i18n"
```

---

### `i18n/en/<app-id>.ftl`

**Purpose:** Source-of-truth English strings. Every user-visible string in the
UI lives here behind an `fl!()` lookup.

**Initial contents:**

```fluent
app-name = <App Name>
welcome  = Welcome
```

---

### `data/<app-id>.desktop`

**Purpose:** XDG desktop entry so the app shows up in the launcher / app
library after install.

**Contents (sketch):**

```ini
[Desktop Entry]
Name=<App Name>
Comment=<short description from Figma>
Exec=<binary-name>
Icon=<app-id>
Type=Application
Categories=Utility;
StartupNotify=true
```

---

### `data/<app-id>.metainfo.xml`

**Purpose:** AppStream metadata so the app appears correctly in COSMIC Store /
GNOME Software with screenshots, summary, and license.

---

### `data/icons/hicolor/scalable/apps/<app-id>.svg`

**Purpose:** Application icon. Placeholder SVG at scaffold time; replaced with
the icon exported from Figma.

---

### `justfile`

**Purpose:** Build / install / uninstall recipes that mirror the COSMIC app
convention — `just build-release`, `just install`, `just vendor`, etc. Drops
binary into `/usr/bin`, desktop file into `/usr/share/applications`, icon
into `/usr/share/icons/hicolor/scalable/apps`.

---

### `README.md`

**Purpose:** What the app is, how to build it, how to install it, where to
file bugs. One screenshot once a screen exists.

---

### `LICENSE`

**Purpose:** OSI-approved license text. COSMIC ecosystem default is GPL-3.0;
confirm preference before committing.

---

## Suggested commit sequence

1. `Cargo.toml`, `LICENSE`, `README.md`, `.gitignore` — repo bones.
2. `src/main.rs`, `src/app.rs` with empty `view()` returning a `text` widget
   — proves the runtime links.
3. `localize.rs` + `i18n.toml` + `i18n/en/<app>.ftl` — first `fl!()` call.
4. `pages/mod.rs` + `pages/home.rs` — page-routing pattern in place.
5. `config.rs` wired into `app.rs` — even if `Config` is empty.
6. `data/` files + `justfile` — installable on a real system.
7. **Stop.** Do not add real widgets until Figma frames are mapped to pages.

## What I need from you to refine this

- A frame list (or screenshot dump) from the Figma so screen→`pages/*.rs`
  mapping is concrete.
- The intended `APP_ID` (reverse-DNS), binary name, and human-readable name.
- Confirmation of windowed-app vs. applet, and sidebar vs. tabs vs. single
  page.
- License preference and target distros (affects `justfile` install paths).
