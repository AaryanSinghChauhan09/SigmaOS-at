// SPDX-License-Identifier: Apache-2.0
//! SigmaOS-at — accessibility tools manager.
//!
//! Manages assistive-technology settings: screen-reader speech
//! (rate/pitch/volume), screen magnifier zoom, high-contrast and
//! colorblind display filters, sticky/repeat/bounce keys, reduced
//! motion, keyboard navigation. Renders AT-SPI/GSettings-style config.
//! JSON persistence.

use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ColorblindFilter { None, Protanopia, Deuteranopia, Tritanopia }
impl ColorblindFilter {
    pub fn label(self) -> &'static str {
        match self { Self::None=>"none", Self::Protanopia=>"protanopia", Self::Deuteranopia=>"deuteranopia", Self::Tritanopia=>"tritanopia" }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s { "none"=>Some(Self::None), "protanopia"=>Some(Self::Protanopia),
            "deuteranopia"=>Some(Self::Deuteranopia), "tritanopia"=>Some(Self::Tritanopia), _=>None }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AccessibilitySettings {
    // screen reader
    pub screen_reader: bool,
    pub speech_rate: u32,          // words per minute 50-500
    pub speech_pitch: u32,         // 0-100 (%)
    pub speech_volume: u32,        // 0-100 (%)
    pub announce_typing: bool,     // speak typed chars
    // magnifier
    pub magnifier: bool,
    pub zoom: u32,                 // percent 100-1600
    // display
    pub high_contrast: bool,
    pub colorblind: ColorblindFilter,
    pub reduced_motion: bool,
    pub large_text: bool,          // font scaling
    // keyboard
    pub sticky_keys: bool,
    pub slow_keys: bool,           // key press acceptance delay
    pub bounce_keys: bool,
    pub mouse_keys: bool,          // control pointer via keypad
    pub keyboard_nav: bool,        // tab through all controls
}

impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self { screen_reader: false, speech_rate: 200, speech_pitch: 50, speech_volume: 100,
            announce_typing: false, magnifier: false, zoom: 100, high_contrast: false,
            colorblind: ColorblindFilter::None, reduced_motion: false, large_text: false,
            sticky_keys: false, slow_keys: false, bounce_keys: false, mouse_keys: false,
            keyboard_nav: false }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AtError {
    BadValue(String, String),
    NoSuchKey(String),
}
impl fmt::Display for AtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadValue(k,v)=>write!(f,"bad value {k}={v}"),
            Self::NoSuchKey(k)=>write!(f,"unknown setting {k}"),
        }
    }
}
impl std::error::Error for AtError {}

/// A named accessibility profile (preset).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AtProfile {
    pub name: String,
    pub settings: AccessibilitySettings,
}

pub struct SigmaOSAccessibility {
    pub settings: AccessibilitySettings,
    profiles: BTreeMap<String, AtProfile>,
}

impl SigmaOSAccessibility {
    pub fn new() -> Self {
        let mut m = Self { settings: AccessibilitySettings::default(), profiles: BTreeMap::new() };
        m.seed_profiles();
        m
    }

    fn seed_profiles(&mut self) {
        let mut low_vision = AccessibilitySettings::default();
        low_vision.magnifier = true; low_vision.zoom = 200; low_vision.large_text = true;
        low_vision.high_contrast = true; low_vision.reduced_motion = true;
        self.profiles.insert("low-vision".into(), AtProfile { name: "low-vision".into(), settings: low_vision });

        let mut blind = AccessibilitySettings::default();
        blind.screen_reader = true; blind.keyboard_nav = true; blind.sticky_keys = true;
        blind.slow_keys = true; blind.announce_typing = true;
        self.profiles.insert("screen-reader".into(), AtProfile { name: "screen-reader".into(), settings: blind });

        let mut motor = AccessibilitySettings::default();
        motor.sticky_keys = true; motor.slow_keys = true; motor.bounce_keys = true;
        motor.mouse_keys = true;
        self.profiles.insert("motor".into(), AtProfile { name: "motor".into(), settings: motor });
    }

    pub fn profile(&self, name: &str) -> Option<&AtProfile> { self.profiles.get(name) }
    pub fn profiles(&self) -> impl Iterator<Item = &AtProfile> { self.profiles.values() }

    /// Apply a preset profile.
    pub fn apply_profile(&mut self, name: &str) -> Result<(), AtError> {
        let p = self.profiles.get(name).ok_or_else(|| AtError::NoSuchKey(name.into()))?;
        self.settings = p.settings.clone();
        Ok(())
    }

    /// Set any setting by dotted key (e.g. "speech.rate", "display.zoom").
    pub fn set(&mut self, key: &str, value: &str) -> Result<(), AtError> {
        let s = &mut self.settings;
        let bool_val = |v: &str| -> Result<bool, AtError> {
            match v { "true"|"1"|"on"|"yes" => Ok(true), "false"|"0"|"off"|"no" => Ok(false),
                _ => Err(AtError::BadValue(key.into(), v.into())) }
        };
        let u32_val = |v: &str, lo: u32, hi: u32| -> Result<u32, AtError> {
            let n: u32 = v.parse().map_err(|_| AtError::BadValue(key.into(), v.into()))?;
            if n < lo || n > hi { return Err(AtError::BadValue(key.into(), v.into())); }
            Ok(n)
        };
        match key {
            "screen_reader" => s.screen_reader = bool_val(value)?,
            "speech.rate" => s.speech_rate = u32_val(value, 50, 500)?,
            "speech.pitch" => s.speech_pitch = u32_val(value, 0, 100)?,
            "speech.volume" => s.speech_volume = u32_val(value, 0, 100)?,
            "announce_typing" => s.announce_typing = bool_val(value)?,
            "magnifier" => s.magnifier = bool_val(value)?,
            "display.zoom" => s.zoom = u32_val(value, 100, 1600)?,
            "high_contrast" => s.high_contrast = bool_val(value)?,
            "colorblind" => s.colorblind = ColorblindFilter::parse(value)
                .ok_or_else(|| AtError::BadValue(key.into(), value.into()))?,
            "reduced_motion" => s.reduced_motion = bool_val(value)?,
            "large_text" => s.large_text = bool_val(value)?,
            "sticky_keys" => s.sticky_keys = bool_val(value)?,
            "slow_keys" => s.slow_keys = bool_val(value)?,
            "bounce_keys" => s.bounce_keys = bool_val(value)?,
            "mouse_keys" => s.mouse_keys = bool_val(value)?,
            "keyboard_nav" => s.keyboard_nav = bool_val(value)?,
            _ => return Err(AtError::NoSuchKey(key.into())),
        }
        Ok(())
    }

    /// Convenience: enable/disable the big features.
    pub fn toggle(&mut self, feature: &str) -> Result<bool, AtError> {
        match feature {
            "screen_reader" => { self.settings.screen_reader = !self.settings.screen_reader; Ok(self.settings.screen_reader) }
            "magnifier" => { self.settings.magnifier = !self.settings.magnifier; Ok(self.settings.magnifier) }
            "high_contrast" => { self.settings.high_contrast = !self.settings.high_contrast; Ok(self.settings.high_contrast) }
            "reduced_motion" => { self.settings.reduced_motion = !self.settings.reduced_motion; Ok(self.settings.reduced_motion) }
            "sticky_keys" => { self.settings.sticky_keys = !self.settings.sticky_keys; Ok(self.settings.sticky_keys) }
            _ => Err(AtError::NoSuchKey(feature.into())),
        }
    }

    /// Count of active accommodations.
    pub fn active_count(&self) -> usize {
        let s = &self.settings;
        [
            s.screen_reader, s.magnifier, s.high_contrast, s.reduced_motion,
            s.large_text, s.sticky_keys, s.slow_keys, s.bounce_keys,
            s.mouse_keys, s.keyboard_nav, s.colorblind != ColorblindFilter::None,
            s.announce_typing,
        ].iter().filter(|&&b| b).count()
    }

    /// Render AT-SPI / GSettings-style config.
    pub fn render_config(&self) -> String {
        let s = &self.settings;
        format!("[sigmaos.a11y]
screen-reader-enabled={}
speech-rate={}
speech-pitch={}
speech-volume={}
announce-typing={}
magnifier-enabled={}
zoom={}
high-contrast={}
colorblind-filter={}
reduced-motion={}
large-text={}
sticky-keys={}
slow-keys={}
bounce-keys={}
mouse-keys={}
keyboard-nav={}
",
            yn(s.screen_reader), s.speech_rate, s.speech_pitch, s.speech_volume,
            yn(s.announce_typing), yn(s.magnifier), s.zoom, yn(s.high_contrast),
            s.colorblind.label(), yn(s.reduced_motion), yn(s.large_text),
            yn(s.sticky_keys), yn(s.slow_keys), yn(s.bounce_keys),
            yn(s.mouse_keys), yn(s.keyboard_nav))
    }

    /// Render a shell-exportable env snippet (e.g. for apps reading env).
    pub fn render_env(&self) -> String {
        let s = &self.settings;
        let mut out = String::new();
        if s.screen_reader { out.push_str("export SIGMAOS_SCREEN_READER=1\n"); }
        if s.magnifier { out.push_str("export SIGMAOS_MAGNIFIER=1\n"); }
        if s.zoom != 100 { out.push_str(&format!("export SIGMAOS_ZOOM={}\n", s.zoom)); }
        if s.high_contrast { out.push_str("export SIGMAOS_HIGH_CONTRAST=1\n"); }
        if s.reduced_motion { out.push_str("export SIGMAOS_REDUCED_MOTION=1\n"); }
        if s.large_text { out.push_str("export SIGMAOS_LARGE_TEXT=1\n"); }
        out
    }

    /// Persist settings to JSON.
    pub fn save_to(&self, path: &std::path::Path) -> Result<(), String> {
        let j = serde_json::to_string_pretty(&self.settings).map_err(|e| e.to_string())?;
        if let Some(p) = path.parent() { std::fs::create_dir_all(p).map_err(|e| e.to_string())?; }
        std::fs::write(path, j).map_err(|e| e.to_string())
    }

    pub fn load_from(path: &std::path::Path) -> Result<Self, String> {
        let txt = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let st: AccessibilitySettings = serde_json::from_str(&txt).map_err(|e| e.to_string())?;
        let mut m = Self::new();
        m.settings = st;
        Ok(m)
    }

    pub fn state_path() -> std::path::PathBuf {
        if let Ok(p) = std::env::var("SIGMAOS_A11Y_FILE") { return std::path::PathBuf::from(p); }
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        std::path::PathBuf::from(home).join(".config/sigmaos/accessibility.json")
    }

    pub fn status_line(&self) -> String {
        format!("accessibility: {} accommodations active{}",
            self.active_count(),
            if self.settings.screen_reader { " (screen reader on)" } else { "" })
    }
}

fn yn(b: bool) -> &'static str { if b { "true" } else { "false" } }

impl Default for SigmaOSAccessibility { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_off() {
        let m = SigmaOSAccessibility::new();
        assert!(!m.settings.screen_reader);
        assert_eq!(m.settings.speech_rate, 200);
        assert_eq!(m.settings.zoom, 100);
        assert_eq!(m.active_count(), 0);
    }

    #[test]
    fn set_values_and_validation() {
        let mut m = SigmaOSAccessibility::new();
        m.set("speech.rate", "300").unwrap();
        assert_eq!(m.settings.speech_rate, 300);
        assert!(m.set("speech.rate", "10").is_err());  // <50
        assert!(m.set("speech.rate", "999").is_err()); // >500
        assert!(m.set("display.zoom", "200").is_ok());
        assert_eq!(m.settings.zoom, 200);
        assert!(m.set("display.zoom", "50").is_err()); // <100
        assert!(m.set("colorblind", "deuteranopia").is_ok());
        assert_eq!(m.settings.colorblind, ColorblindFilter::Deuteranopia);
        assert!(m.set("colorblind", "bogus").is_err());
        assert!(m.set("nonexistent", "1").is_err());
        assert!(m.set("high_contrast", "maybe").is_err());
    }

    #[test]
    fn toggles() {
        let mut m = SigmaOSAccessibility::new();
        assert!(m.toggle("screen_reader").unwrap());
        assert!(m.settings.screen_reader);
        assert!(!m.toggle("screen_reader").unwrap());
        assert!(m.toggle("magnifier").is_ok());
        assert!(m.toggle("bogus").is_err());
    }

    #[test]
    fn profiles_apply() {
        let mut m = SigmaOSAccessibility::new();
        m.apply_profile("low-vision").unwrap();
        assert!(m.settings.magnifier);
        assert_eq!(m.settings.zoom, 200);
        assert!(m.settings.high_contrast);
        m.apply_profile("screen-reader").unwrap();
        assert!(m.settings.screen_reader);
        assert!(m.settings.sticky_keys);
        assert!(m.apply_profile("nope").is_err());
        assert!(m.profiles().count() >= 3);
    }

    #[test]
    fn active_count_tracks() {
        let mut m = SigmaOSAccessibility::new();
        assert_eq!(m.active_count(), 0);
        m.set("high_contrast", "true").unwrap();
        m.set("sticky_keys", "true").unwrap();
        assert_eq!(m.active_count(), 2);
    }

    #[test]
    fn render_outputs() {
        let mut m = SigmaOSAccessibility::new();
        m.set("screen_reader", "true").unwrap();
        m.set("display.zoom", "150").unwrap();
        let cfg = m.render_config();
        assert!(cfg.contains("screen-reader-enabled=true"));
        assert!(cfg.contains("zoom=150"));
        let env = m.render_env();
        assert!(env.contains("SIGMAOS_SCREEN_READER=1"));
        assert!(env.contains("SIGMAOS_ZOOM=150"));
        // nothing enabled -> empty env
        let m2 = SigmaOSAccessibility::new();
        assert!(m2.render_env().is_empty());
    }
}
