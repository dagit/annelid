use serde_derive::{Deserialize, Serialize};

/// Framework-independent key code enum.
/// Variant names match egui::Key's serde representation for config file compatibility.
#[derive(
    Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq, Hash, strum_macros::EnumIter,
)]
pub enum KeyCode {
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    Escape,
    Tab,
    Backspace,
    Enter,
    Space,
    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    F26,
    F27,
    F28,
    F29,
    F30,
    F31,
    F32,
    F33,
    F34,
    F35,
    Minus,
    Plus,
    Equals,
    Copy,
    Cut,
    Paste,
    Colon,
    Comma,
    Backslash,
    Slash,
    Pipe,
    Questionmark,
    OpenBracket,
    CloseBracket,
    Backtick,
    Period,
    Semicolon,
    Quote,
    Exclamationmark,
    OpenCurlyBracket,
    CloseCurlyBracket,
    BrowserBack,
}

/// Framework-independent modifier flags.
/// Field names match egui::Modifiers' serde representation for config file compatibility.
#[derive(Deserialize, Serialize, Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub alt: bool,
    pub ctrl: bool,
    pub shift: bool,
    pub mac_cmd: bool,
    pub command: bool,
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub struct HotKey {
    pub key: KeyCode,
    pub modifiers: Modifiers,
}

impl HotKey {
    pub fn to_livesplit_hotkey(self) -> livesplit_hotkey::Hotkey {
        to_livesplit_keycode(self.key).with_modifiers(to_livesplit_modifiers(self.modifiers))
    }
}

// --- egui conversions ---

impl From<egui::Key> for KeyCode {
    fn from(key: egui::Key) -> Self {
        match key {
            egui::Key::ArrowDown => KeyCode::ArrowDown,
            egui::Key::ArrowLeft => KeyCode::ArrowLeft,
            egui::Key::ArrowRight => KeyCode::ArrowRight,
            egui::Key::ArrowUp => KeyCode::ArrowUp,
            egui::Key::Escape => KeyCode::Escape,
            egui::Key::Tab => KeyCode::Tab,
            egui::Key::Backspace => KeyCode::Backspace,
            egui::Key::Enter => KeyCode::Enter,
            egui::Key::Space => KeyCode::Space,
            egui::Key::Insert => KeyCode::Insert,
            egui::Key::Delete => KeyCode::Delete,
            egui::Key::Home => KeyCode::Home,
            egui::Key::End => KeyCode::End,
            egui::Key::PageUp => KeyCode::PageUp,
            egui::Key::PageDown => KeyCode::PageDown,
            egui::Key::Num0 => KeyCode::Num0,
            egui::Key::Num1 => KeyCode::Num1,
            egui::Key::Num2 => KeyCode::Num2,
            egui::Key::Num3 => KeyCode::Num3,
            egui::Key::Num4 => KeyCode::Num4,
            egui::Key::Num5 => KeyCode::Num5,
            egui::Key::Num6 => KeyCode::Num6,
            egui::Key::Num7 => KeyCode::Num7,
            egui::Key::Num8 => KeyCode::Num8,
            egui::Key::Num9 => KeyCode::Num9,
            egui::Key::A => KeyCode::A,
            egui::Key::B => KeyCode::B,
            egui::Key::C => KeyCode::C,
            egui::Key::D => KeyCode::D,
            egui::Key::E => KeyCode::E,
            egui::Key::F => KeyCode::F,
            egui::Key::G => KeyCode::G,
            egui::Key::H => KeyCode::H,
            egui::Key::I => KeyCode::I,
            egui::Key::J => KeyCode::J,
            egui::Key::K => KeyCode::K,
            egui::Key::L => KeyCode::L,
            egui::Key::M => KeyCode::M,
            egui::Key::N => KeyCode::N,
            egui::Key::O => KeyCode::O,
            egui::Key::P => KeyCode::P,
            egui::Key::Q => KeyCode::Q,
            egui::Key::R => KeyCode::R,
            egui::Key::S => KeyCode::S,
            egui::Key::T => KeyCode::T,
            egui::Key::U => KeyCode::U,
            egui::Key::V => KeyCode::V,
            egui::Key::W => KeyCode::W,
            egui::Key::X => KeyCode::X,
            egui::Key::Y => KeyCode::Y,
            egui::Key::Z => KeyCode::Z,
            egui::Key::F1 => KeyCode::F1,
            egui::Key::F2 => KeyCode::F2,
            egui::Key::F3 => KeyCode::F3,
            egui::Key::F4 => KeyCode::F4,
            egui::Key::F5 => KeyCode::F5,
            egui::Key::F6 => KeyCode::F6,
            egui::Key::F7 => KeyCode::F7,
            egui::Key::F8 => KeyCode::F8,
            egui::Key::F9 => KeyCode::F9,
            egui::Key::F10 => KeyCode::F10,
            egui::Key::F11 => KeyCode::F11,
            egui::Key::F12 => KeyCode::F12,
            egui::Key::F13 => KeyCode::F13,
            egui::Key::F14 => KeyCode::F14,
            egui::Key::F15 => KeyCode::F15,
            egui::Key::F16 => KeyCode::F16,
            egui::Key::F17 => KeyCode::F17,
            egui::Key::F18 => KeyCode::F18,
            egui::Key::F19 => KeyCode::F19,
            egui::Key::F20 => KeyCode::F20,
            egui::Key::F21 => KeyCode::F21,
            egui::Key::F22 => KeyCode::F22,
            egui::Key::F23 => KeyCode::F23,
            egui::Key::F24 => KeyCode::F24,
            egui::Key::F25 => KeyCode::F25,
            egui::Key::F26 => KeyCode::F26,
            egui::Key::F27 => KeyCode::F27,
            egui::Key::F28 => KeyCode::F28,
            egui::Key::F29 => KeyCode::F29,
            egui::Key::F30 => KeyCode::F30,
            egui::Key::F31 => KeyCode::F31,
            egui::Key::F32 => KeyCode::F32,
            egui::Key::F33 => KeyCode::F33,
            egui::Key::F34 => KeyCode::F34,
            egui::Key::F35 => KeyCode::F35,
            egui::Key::Minus => KeyCode::Minus,
            egui::Key::Plus => KeyCode::Plus,
            egui::Key::Equals => KeyCode::Equals,
            egui::Key::Copy => KeyCode::Copy,
            egui::Key::Cut => KeyCode::Cut,
            egui::Key::Paste => KeyCode::Paste,
            egui::Key::Colon => KeyCode::Colon,
            egui::Key::Comma => KeyCode::Comma,
            egui::Key::Backslash => KeyCode::Backslash,
            egui::Key::Slash => KeyCode::Slash,
            egui::Key::Pipe => KeyCode::Pipe,
            egui::Key::Questionmark => KeyCode::Questionmark,
            egui::Key::OpenBracket => KeyCode::OpenBracket,
            egui::Key::CloseBracket => KeyCode::CloseBracket,
            egui::Key::Backtick => KeyCode::Backtick,
            egui::Key::Period => KeyCode::Period,
            egui::Key::Semicolon => KeyCode::Semicolon,
            egui::Key::Quote => KeyCode::Quote,
            egui::Key::Exclamationmark => KeyCode::Exclamationmark,
            egui::Key::OpenCurlyBracket => KeyCode::OpenCurlyBracket,
            egui::Key::CloseCurlyBracket => KeyCode::CloseCurlyBracket,
            egui::Key::BrowserBack => KeyCode::BrowserBack,
        }
    }
}

impl From<KeyCode> for egui::Key {
    fn from(key: KeyCode) -> Self {
        match key {
            KeyCode::ArrowDown => egui::Key::ArrowDown,
            KeyCode::ArrowLeft => egui::Key::ArrowLeft,
            KeyCode::ArrowRight => egui::Key::ArrowRight,
            KeyCode::ArrowUp => egui::Key::ArrowUp,
            KeyCode::Escape => egui::Key::Escape,
            KeyCode::Tab => egui::Key::Tab,
            KeyCode::Backspace => egui::Key::Backspace,
            KeyCode::Enter => egui::Key::Enter,
            KeyCode::Space => egui::Key::Space,
            KeyCode::Insert => egui::Key::Insert,
            KeyCode::Delete => egui::Key::Delete,
            KeyCode::Home => egui::Key::Home,
            KeyCode::End => egui::Key::End,
            KeyCode::PageUp => egui::Key::PageUp,
            KeyCode::PageDown => egui::Key::PageDown,
            KeyCode::Num0 => egui::Key::Num0,
            KeyCode::Num1 => egui::Key::Num1,
            KeyCode::Num2 => egui::Key::Num2,
            KeyCode::Num3 => egui::Key::Num3,
            KeyCode::Num4 => egui::Key::Num4,
            KeyCode::Num5 => egui::Key::Num5,
            KeyCode::Num6 => egui::Key::Num6,
            KeyCode::Num7 => egui::Key::Num7,
            KeyCode::Num8 => egui::Key::Num8,
            KeyCode::Num9 => egui::Key::Num9,
            KeyCode::A => egui::Key::A,
            KeyCode::B => egui::Key::B,
            KeyCode::C => egui::Key::C,
            KeyCode::D => egui::Key::D,
            KeyCode::E => egui::Key::E,
            KeyCode::F => egui::Key::F,
            KeyCode::G => egui::Key::G,
            KeyCode::H => egui::Key::H,
            KeyCode::I => egui::Key::I,
            KeyCode::J => egui::Key::J,
            KeyCode::K => egui::Key::K,
            KeyCode::L => egui::Key::L,
            KeyCode::M => egui::Key::M,
            KeyCode::N => egui::Key::N,
            KeyCode::O => egui::Key::O,
            KeyCode::P => egui::Key::P,
            KeyCode::Q => egui::Key::Q,
            KeyCode::R => egui::Key::R,
            KeyCode::S => egui::Key::S,
            KeyCode::T => egui::Key::T,
            KeyCode::U => egui::Key::U,
            KeyCode::V => egui::Key::V,
            KeyCode::W => egui::Key::W,
            KeyCode::X => egui::Key::X,
            KeyCode::Y => egui::Key::Y,
            KeyCode::Z => egui::Key::Z,
            KeyCode::F1 => egui::Key::F1,
            KeyCode::F2 => egui::Key::F2,
            KeyCode::F3 => egui::Key::F3,
            KeyCode::F4 => egui::Key::F4,
            KeyCode::F5 => egui::Key::F5,
            KeyCode::F6 => egui::Key::F6,
            KeyCode::F7 => egui::Key::F7,
            KeyCode::F8 => egui::Key::F8,
            KeyCode::F9 => egui::Key::F9,
            KeyCode::F10 => egui::Key::F10,
            KeyCode::F11 => egui::Key::F11,
            KeyCode::F12 => egui::Key::F12,
            KeyCode::F13 => egui::Key::F13,
            KeyCode::F14 => egui::Key::F14,
            KeyCode::F15 => egui::Key::F15,
            KeyCode::F16 => egui::Key::F16,
            KeyCode::F17 => egui::Key::F17,
            KeyCode::F18 => egui::Key::F18,
            KeyCode::F19 => egui::Key::F19,
            KeyCode::F20 => egui::Key::F20,
            KeyCode::F21 => egui::Key::F21,
            KeyCode::F22 => egui::Key::F22,
            KeyCode::F23 => egui::Key::F23,
            KeyCode::F24 => egui::Key::F24,
            KeyCode::F25 => egui::Key::F25,
            KeyCode::F26 => egui::Key::F26,
            KeyCode::F27 => egui::Key::F27,
            KeyCode::F28 => egui::Key::F28,
            KeyCode::F29 => egui::Key::F29,
            KeyCode::F30 => egui::Key::F30,
            KeyCode::F31 => egui::Key::F31,
            KeyCode::F32 => egui::Key::F32,
            KeyCode::F33 => egui::Key::F33,
            KeyCode::F34 => egui::Key::F34,
            KeyCode::F35 => egui::Key::F35,
            KeyCode::Minus => egui::Key::Minus,
            KeyCode::Plus => egui::Key::Plus,
            KeyCode::Equals => egui::Key::Equals,
            KeyCode::Copy => egui::Key::Copy,
            KeyCode::Cut => egui::Key::Cut,
            KeyCode::Paste => egui::Key::Paste,
            KeyCode::Colon => egui::Key::Colon,
            KeyCode::Comma => egui::Key::Comma,
            KeyCode::Backslash => egui::Key::Backslash,
            KeyCode::Slash => egui::Key::Slash,
            KeyCode::Pipe => egui::Key::Pipe,
            KeyCode::Questionmark => egui::Key::Questionmark,
            KeyCode::OpenBracket => egui::Key::OpenBracket,
            KeyCode::CloseBracket => egui::Key::CloseBracket,
            KeyCode::Backtick => egui::Key::Backtick,
            KeyCode::Period => egui::Key::Period,
            KeyCode::Semicolon => egui::Key::Semicolon,
            KeyCode::Quote => egui::Key::Quote,
            KeyCode::Exclamationmark => egui::Key::Exclamationmark,
            KeyCode::OpenCurlyBracket => egui::Key::OpenCurlyBracket,
            KeyCode::CloseCurlyBracket => egui::Key::CloseCurlyBracket,
            KeyCode::BrowserBack => egui::Key::BrowserBack,
        }
    }
}

impl From<egui::Modifiers> for Modifiers {
    fn from(m: egui::Modifiers) -> Self {
        Modifiers {
            alt: m.alt,
            ctrl: m.ctrl,
            shift: m.shift,
            mac_cmd: m.mac_cmd,
            command: m.command,
        }
    }
}

impl From<Modifiers> for egui::Modifiers {
    fn from(m: Modifiers) -> Self {
        egui::Modifiers {
            alt: m.alt,
            ctrl: m.ctrl,
            shift: m.shift,
            mac_cmd: m.mac_cmd,
            command: m.command,
        }
    }
}

impl HotKey {
    /// Convert to egui key + modifiers for local hotkey matching.
    pub fn to_egui(&self) -> (egui::Modifiers, egui::Key) {
        (self.modifiers.into(), self.key.into())
    }
}

// --- livesplit-hotkey conversions ---

pub fn to_livesplit_keycode(key: KeyCode) -> livesplit_hotkey::KeyCode {
    use livesplit_hotkey::KeyCode::*;

    match key {
        KeyCode::ArrowDown => ArrowDown,
        KeyCode::ArrowLeft => ArrowLeft,
        KeyCode::ArrowRight => ArrowRight,
        KeyCode::ArrowUp => ArrowUp,
        KeyCode::Escape => Escape,
        KeyCode::Tab => Tab,
        KeyCode::Backspace => Backspace,
        KeyCode::Enter => Enter,
        KeyCode::Space => Space,
        KeyCode::Insert => Insert,
        KeyCode::Delete => Delete,
        KeyCode::Home => Home,
        KeyCode::End => End,
        KeyCode::PageUp => PageUp,
        KeyCode::PageDown => PageDown,
        KeyCode::Num0 => Numpad0,
        KeyCode::Num1 => Numpad1,
        KeyCode::Num2 => Numpad2,
        KeyCode::Num3 => Numpad3,
        KeyCode::Num4 => Numpad4,
        KeyCode::Num5 => Numpad5,
        KeyCode::Num6 => Numpad6,
        KeyCode::Num7 => Numpad7,
        KeyCode::Num8 => Numpad8,
        KeyCode::Num9 => Numpad9,
        KeyCode::A => KeyA,
        KeyCode::B => KeyB,
        KeyCode::C => KeyC,
        KeyCode::D => KeyD,
        KeyCode::E => KeyE,
        KeyCode::F => KeyF,
        KeyCode::G => KeyG,
        KeyCode::H => KeyH,
        KeyCode::I => KeyI,
        KeyCode::J => KeyJ,
        KeyCode::K => KeyK,
        KeyCode::L => KeyL,
        KeyCode::M => KeyM,
        KeyCode::N => KeyN,
        KeyCode::O => KeyO,
        KeyCode::P => KeyP,
        KeyCode::Q => KeyQ,
        KeyCode::R => KeyR,
        KeyCode::S => KeyS,
        KeyCode::T => KeyT,
        KeyCode::U => KeyU,
        KeyCode::V => KeyV,
        KeyCode::W => KeyW,
        KeyCode::X => KeyX,
        KeyCode::Y => KeyY,
        KeyCode::Z => KeyZ,
        KeyCode::F1 => F1,
        KeyCode::F2 => F2,
        KeyCode::F3 => F3,
        KeyCode::F4 => F4,
        KeyCode::F5 => F5,
        KeyCode::F6 => F6,
        KeyCode::F7 => F7,
        KeyCode::F8 => F8,
        KeyCode::F9 => F9,
        KeyCode::F10 => F10,
        KeyCode::F11 => F11,
        KeyCode::F12 => F12,
        KeyCode::F13 => F13,
        KeyCode::F14 => F14,
        KeyCode::F15 => F15,
        KeyCode::F16 => F16,
        KeyCode::F17 => F17,
        KeyCode::F18 => F18,
        KeyCode::F19 => F19,
        KeyCode::F20 => F20,
        KeyCode::F21 => F21,
        KeyCode::F22 => F22,
        KeyCode::F23 => F23,
        KeyCode::F24 => F24,
        KeyCode::F25 => F24, // livesplit-hotkey doesn't support F25+
        KeyCode::F26 => F24,
        KeyCode::F27 => F24,
        KeyCode::F28 => F24,
        KeyCode::F29 => F24,
        KeyCode::F30 => F24,
        KeyCode::F31 => F24,
        KeyCode::F32 => F24,
        KeyCode::F33 => F24,
        KeyCode::F34 => F24,
        KeyCode::F35 => F24,
        KeyCode::Minus => Minus,
        KeyCode::Plus => Equal,
        KeyCode::Equals => Equal,
        KeyCode::Copy => Copy,
        KeyCode::Cut => Cut,
        KeyCode::Paste => Paste,
        KeyCode::Colon => Semicolon,
        KeyCode::Comma => Comma,
        KeyCode::Backslash => Backslash,
        KeyCode::Slash => Slash,
        KeyCode::Pipe => IntlBackslash,
        KeyCode::Questionmark => Slash,
        KeyCode::OpenBracket => BracketLeft,
        KeyCode::CloseBracket => BracketRight,
        KeyCode::Backtick => Backquote,
        KeyCode::Period => Period,
        KeyCode::Semicolon => Semicolon,
        KeyCode::Quote => Quote,
        KeyCode::Exclamationmark => Digit1,
        KeyCode::OpenCurlyBracket => BracketLeft,
        KeyCode::CloseCurlyBracket => BracketRight,
        KeyCode::BrowserBack => BrowserBack,
    }
}

pub fn to_livesplit_keycode_alternative(key: KeyCode) -> Option<livesplit_hotkey::KeyCode> {
    use livesplit_hotkey::KeyCode::*;

    match key {
        KeyCode::Num0 => Some(Digit0),
        KeyCode::Num1 => Some(Digit1),
        KeyCode::Num2 => Some(Digit2),
        KeyCode::Num3 => Some(Digit3),
        KeyCode::Num4 => Some(Digit4),
        KeyCode::Num5 => Some(Digit5),
        KeyCode::Num6 => Some(Digit6),
        KeyCode::Num7 => Some(Digit7),
        KeyCode::Num8 => Some(Digit8),
        KeyCode::Num9 => Some(Digit9),
        _ => None,
    }
}

pub fn to_livesplit_modifiers(modifiers: Modifiers) -> livesplit_hotkey::Modifiers {
    use livesplit_hotkey::Modifiers as LM;
    let mut mods = LM::empty();
    if modifiers.shift {
        mods.insert(LM::SHIFT);
    }
    if modifiers.ctrl {
        mods.insert(LM::CONTROL);
    }
    if modifiers.alt {
        mods.insert(LM::ALT);
    }
    if modifiers.mac_cmd || modifiers.command {
        mods.insert(LM::META);
    }
    mods
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_round_trip() {
        // Simulate what a saved settings.toml hotkey section looks like
        let toml_str = r#"
key = "Num1"

[modifiers]
alt = false
ctrl = false
shift = false
mac_cmd = false
command = false
"#;
        let hk: HotKey = toml::from_str(toml_str).expect("should deserialize");
        assert_eq!(hk.key, KeyCode::Num1);
        assert_eq!(hk.modifiers, Modifiers::default());

        // Re-serialize and check it matches
        let reserialized = toml::to_string_pretty(&hk).expect("should serialize");
        let hk2: HotKey = toml::from_str(&reserialized).expect("should round-trip");
        assert_eq!(hk.key, hk2.key);
        assert_eq!(hk.modifiers, hk2.modifiers);
    }

    #[test]
    fn all_default_hotkeys_round_trip() {
        // Test the full config with all default hotkeys
        let config = crate::config::app_config::AppConfig::default();
        let serialized = toml::to_string_pretty(&config).expect("should serialize");
        let deserialized: crate::config::app_config::AppConfig =
            toml::from_str(&serialized).expect("should deserialize");

        // Verify hotkeys survived
        assert_eq!(
            config.hot_key_start.unwrap().key,
            deserialized.hot_key_start.unwrap().key
        );
        assert_eq!(
            config.hot_key_reset.unwrap().key,
            deserialized.hot_key_reset.unwrap().key
        );
    }
}
