use serde::{Deserialize, Serialize};
use zvariant::Type;

/// Best engine rank.
pub const ENGINE_RANK_BEST: u32 = 0;
/// Good engine rank.
pub const ENGINE_RANK_GOOD: u32 = 1;
/// Normal engine rank.
pub const ENGINE_RANK_NORMAL: u32 = 2;
/// Bad engine rank.
pub const ENGINE_RANK_BAD: u32 = 3;
/// Worst engine rank.
pub const ENGINE_RANK_WORST: u32 = 4;

/// Description of a single input method engine within a component.
///
/// Passed to the ibus-daemon during component registration so the panel can
/// display the engine in its switcher UI.
///
/// # Example
///
/// ```
/// use libibus_rs::EngineDesc;
///
/// let engine = EngineDesc::new("my-engine", "My Engine", "Description", "ja");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct EngineDesc {
    /// Unique engine name (e.g. `libibus-rs-demo`).
    pub name: String,
    /// Human-readable display name.
    pub longname: String,
    /// Descriptive text.
    pub description: String,
    /// Language code (e.g. `ja`, `zh_CN`).
    pub language: String,
    /// SPDX license identifier.
    pub license: String,
    /// Author name.
    pub author: String,
    /// Icon file name (without path).
    pub icon: String,
    /// Default keyboard layout (e.g. `us`, `jp`).
    pub layout: String,
    /// Hotkeys to switch to this engine.
    pub hotkeys: Vec<String>,
    /// Display priority (see `ENGINE_RANK_*` constants).
    pub rank: u32,
    /// Symbol displayed in the panel (e.g. `あ`).
    pub symbol: String,
    /// Path to a setup tool executable.
    pub setup: String,
    /// Keyboard layout variant.
    pub layout_variants: String,
    /// Keyboard layout option.
    pub layout_option: String,
    /// Engine version string.
    pub version: String,
    /// Gettext text domain for translations.
    pub text_domain: String,
}

impl EngineDesc {
    /// Create a new engine description with the minimum required fields.
    pub fn new(name: &str, longname: &str, description: &str, language: &str) -> Self {
        Self {
            name: name.to_owned(),
            longname: longname.to_owned(),
            description: description.to_owned(),
            language: language.to_owned(),
            license: String::new(),
            author: String::new(),
            icon: String::new(),
            layout: "us".to_owned(),
            hotkeys: Vec::new(),
            rank: ENGINE_RANK_NORMAL,
            symbol: String::new(),
            setup: String::new(),
            layout_variants: String::new(),
            layout_option: String::new(),
            version: "0.1.0".to_owned(),
            text_domain: String::new(),
        }
    }

    /// Set the engine license.
    pub fn set_license(&mut self, license: &str) -> &mut Self {
        self.license = license.to_owned();
        self
    }

    /// Set the engine author.
    pub fn set_author(&mut self, author: &str) -> &mut Self {
        self.author = author.to_owned();
        self
    }

    /// Set the engine icon name.
    pub fn set_icon(&mut self, icon: &str) -> &mut Self {
        self.icon = icon.to_owned();
        self
    }

    /// Set the default keyboard layout.
    pub fn set_layout(&mut self, layout: &str) -> &mut Self {
        self.layout = layout.to_owned();
        self
    }

    /// Set the hotkeys that activate this engine.
    pub fn set_hotkeys(&mut self, hotkeys: Vec<&str>) -> &mut Self {
        self.hotkeys = hotkeys.into_iter().map(|s| s.to_owned()).collect();
        self
    }

    /// Set the display rank.
    pub fn set_rank(&mut self, rank: u32) -> &mut Self {
        self.rank = rank;
        self
    }

    /// Set the panel symbol.
    pub fn set_symbol(&mut self, symbol: &str) -> &mut Self {
        self.symbol = symbol.to_owned();
        self
    }

    /// Set the setup tool path.
    pub fn set_setup(&mut self, setup: &str) -> &mut Self {
        self.setup = setup.to_owned();
        self
    }

    /// Set the engine version.
    pub fn set_version(&mut self, version: &str) -> &mut Self {
        self.version = version.to_owned();
        self
    }
}

/// An IBus component, representing one or more input method engines.
///
/// A component corresponds to an `.xml` file that the ibus-daemon reads to
/// discover available engines, or that you register at runtime via
/// [`Bus::register_component`](crate::Bus::register_component).
///
/// # Example
///
/// ```
/// use libibus_rs::{Component, EngineDesc};
///
/// let mut component = Component::new(
///     "com.example.MyEngine",
///     "My IME package",
///     "1.0",
///     "MIT",
///     "Author",
///     "https://example.com",
///     "/usr/libexec/ibus-engine-my",
/// );
/// component.add_engine(EngineDesc::new("my-eng", "My Engine", "Desc", "en"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Component {
    /// Unique component name (e.g. `com.example.MyEngine`).
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Component version.
    pub version: String,
    /// SPDX license identifier.
    pub license: String,
    /// Author name.
    pub author: String,
    /// Homepage URL.
    pub homepage: String,
    /// Gettext text domain.
    pub text_domain: String,
    /// Path to the engine executable.
    pub exec_path: String,
    /// Arguments passed to the executable.
    pub exec_args: Vec<String>,
    /// Engines provided by this component.
    pub engines: Vec<EngineDesc>,
    /// Paths to watch for changes.
    pub watch_paths: Vec<String>,
}

impl Component {
    /// Create a new component.
    pub fn new(
        name: &str,
        description: &str,
        version: &str,
        license: &str,
        author: &str,
        homepage: &str,
        exec_path: &str,
    ) -> Self {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            version: version.to_owned(),
            license: license.to_owned(),
            author: author.to_owned(),
            homepage: homepage.to_owned(),
            text_domain: String::new(),
            exec_path: exec_path.to_owned(),
            exec_args: Vec::new(),
            engines: Vec::new(),
            watch_paths: Vec::new(),
        }
    }

    /// Add an engine to this component.
    pub fn add_engine(&mut self, engine: EngineDesc) -> &mut Self {
        self.engines.push(engine);
        self
    }

    /// Set the executable arguments.
    pub fn set_exec_args(&mut self, args: Vec<&str>) -> &mut Self {
        self.exec_args = args.into_iter().map(|s| s.to_owned()).collect();
        self
    }

    /// Set the gettext text domain.
    pub fn set_text_domain(&mut self, text_domain: &str) -> &mut Self {
        self.text_domain = text_domain.to_owned();
        self
    }

    /// Add a path to watch.
    pub fn add_watch_path(&mut self, path: &str) -> &mut Self {
        self.watch_paths.push(path.to_owned());
        self
    }
}
