//! User settings, persisted as simple `key=value` lines (no serde needed).
use crate::files;
use gpui::Global;
use std::path::PathBuf;

pub const FONTS: [&str; 5] = ["Menlo", "Helvetica", "Times New Roman", "Courier New", "Monaco"];
pub const SIZES: [f32; 10] = [10., 12., 14., 16., 18., 20., 24., 28., 32., 48.];
const MAX_RECENT: usize = 10;

pub struct Settings {
    pub font_family: String,
    pub font_size: f32,
    pub theme: String,
    pub wrap: bool,
    pub line_numbers: bool,
    pub check_updates: bool,
    pub recent: Vec<PathBuf>,
}

impl Global for Settings {}

impl Default for Settings {
    fn default() -> Self {
        Self {
            font_family: FONTS[0].into(),
            font_size: 16.,
            theme: "Dark+".into(),
            wrap: true,
            line_numbers: true,
            check_updates: true,
            recent: Vec::new(),
        }
    }
}

impl Settings {
    fn file() -> PathBuf {
        files::config_dir().join("settings.conf")
    }

    pub fn load() -> Self {
        let mut s = Self::default();
        let Ok(raw) = std::fs::read_to_string(Self::file()) else { return s };
        let legacy_seen = raw.lines().any(|l| l.starts_with("theme="));
        for line in raw.lines() {
            let Some((k, v)) = line.split_once('=') else { continue };
            match k {
                "font" => s.font_family = v.into(),
                "size" => s.font_size = v.parse().unwrap_or(s.font_size).clamp(8., 72.),
                "theme" => s.theme = v.into(),
                "dark" if !legacy_seen => s.theme = if v == "1" { "Dark+" } else { "Light+" }.into(),
                "wrap" => s.wrap = v == "1",
                "lines" => s.line_numbers = v == "1",
                "updates" => s.check_updates = v == "1",
                "recent" => s.recent.push(v.into()),
                _ => {}
            }
        }
        s
    }

    pub fn save(&self) {
        let mut out = format!(
            "font={}\nsize={}\ntheme={}\nwrap={}\nlines={}\nupdates={}\n",
            self.font_family,
            self.font_size,
            self.theme,
            self.wrap as u8,
            self.line_numbers as u8,
            self.check_updates as u8
        );
        for p in &self.recent {
            out.push_str(&format!("recent={}\n", p.display()));
        }
        let _ = std::fs::create_dir_all(files::config_dir());
        let _ = std::fs::write(Self::file(), out);
    }

    pub fn push_recent(&mut self, p: PathBuf) {
        self.recent.retain(|r| r != &p);
        self.recent.insert(0, p);
        self.recent.truncate(MAX_RECENT);
    }

    pub fn zoom(&mut self, d: f32) {
        self.font_size = (self.font_size + d).clamp(8., 72.);
    }
}
