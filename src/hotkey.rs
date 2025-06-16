use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub struct HotKey {
    pub key: ::egui::Key,
    pub modifiers: ::egui::Modifiers,
}

impl HotKey {
    pub fn to_livesplit_hotkey(self) -> livesplit_hotkey::Hotkey {
        to_livesplit_keycode(&self.key).with_modifiers(to_livesplit_modifiers(&self.modifiers))
    }
}

pub fn to_livesplit_keycode(key: &::egui::Key) -> livesplit_hotkey::KeyCode {
    use livesplit_hotkey::KeyCode::*;

    match key {
        egui::Key::ArrowDown => ArrowDown,
        egui::Key::ArrowLeft => ArrowLeft,
        egui::Key::ArrowRight => ArrowRight,
        egui::Key::ArrowUp => ArrowUp,
        egui::Key::Escape => Escape,
        egui::Key::Tab => Tab,
        egui::Key::Backspace => Backspace,
        egui::Key::Enter => Enter,
        egui::Key::Space => Space,
        egui::Key::Insert => Insert,
        egui::Key::Delete => Delete,
        egui::Key::Home => Home,
        egui::Key::End => End,
        egui::Key::PageUp => PageUp,
        egui::Key::PageDown => PageDown,
        egui::Key::Num0 => Numpad0,
        egui::Key::Num1 => Numpad1,
        egui::Key::Num2 => Numpad2,
        egui::Key::Num3 => Numpad3,
        egui::Key::Num4 => Numpad4,
        egui::Key::Num5 => Numpad5,
        egui::Key::Num6 => Numpad6,
        egui::Key::Num7 => Numpad7,
        egui::Key::Num8 => Numpad8,
        egui::Key::Num9 => Numpad9,
        egui::Key::A => KeyA,
        egui::Key::B => KeyB,
        egui::Key::C => KeyC,
        egui::Key::D => KeyD,
        egui::Key::E => KeyE,
        egui::Key::F => KeyF,
        egui::Key::G => KeyG,
        egui::Key::H => KeyH,
        egui::Key::I => KeyI,
        egui::Key::J => KeyJ,
        egui::Key::K => KeyK,
        egui::Key::L => KeyL,
        egui::Key::M => KeyM,
        egui::Key::N => KeyN,
        egui::Key::O => KeyO,
        egui::Key::P => KeyP,
        egui::Key::Q => KeyQ,
        egui::Key::R => KeyR,
        egui::Key::S => KeyS,
        egui::Key::T => KeyT,
        egui::Key::U => KeyU,
        egui::Key::V => KeyV,
        egui::Key::W => KeyW,
        egui::Key::X => KeyX,
        egui::Key::Y => KeyY,
        egui::Key::Z => KeyZ,
        egui::Key::F1 => F1,
        egui::Key::F2 => F2,
        egui::Key::F3 => F3,
        egui::Key::F4 => F4,
        egui::Key::F5 => F5,
        egui::Key::F6 => F6,
        egui::Key::F7 => F7,
        egui::Key::F8 => F8,
        egui::Key::F9 => F9,
        egui::Key::F10 => F10,
        egui::Key::F11 => F11,
        egui::Key::F12 => F12,
        egui::Key::F13 => F13,
        egui::Key::F14 => F14,
        egui::Key::F15 => F15,
        egui::Key::F16 => F16,
        egui::Key::F17 => F17,
        egui::Key::F18 => F18,
        egui::Key::F19 => F19,
        egui::Key::F20 => F20,
        egui::Key::F21 => F21,
        egui::Key::F22 => F22,
        egui::Key::F23 => F23,
        egui::Key::F24 => F24,
        egui::Key::F25 => F24, // TODO: hotkey lib doesn't support this yet
        egui::Key::F26 => F24, // TODO: hotkey lib doesn't support this yet
        egui::Key::F27 => F24, // TODO: hotkey lib doesn't support this yet
        egui::Key::F28 => F24, // TODO: hotkey lib doesn't support this yet
        egui::Key::F29 => F24, // TODO: hotkey lib doesn't support this yet
        egui::Key::F30 => F24, // TODO: hotkey lib doesn't support this yet
        egui::Key::F31 => F24, // TODO: hotkey lib doesn't support this yet
        egui::Key::F32 => F24, // TODO: hotkey lib doesn't support this yet
        egui::Key::F33 => F24, // TODO: hotkey lib doesn't support this yet
        egui::Key::F34 => F24, // TODO: hotkey lib doesn't support this yet
        egui::Key::F35 => F24, // TODO: hotkey lib doesn't support this yet
        egui::Key::Minus => Minus,
        egui::Key::Plus => Equal,
        egui::Key::Equals => Equal,
        egui::Key::Copy => Copy,
        egui::Key::Cut => Cut,
        egui::Key::Paste => Paste,
        egui::Key::Colon => Semicolon,
        egui::Key::Comma => Comma,
        egui::Key::Backslash => Backslash,
        egui::Key::Slash => Slash,
        egui::Key::Pipe => IntlBackslash,
        egui::Key::Questionmark => Slash,
        egui::Key::OpenBracket => BracketLeft,
        egui::Key::CloseBracket => BracketRight,
        egui::Key::Backtick => Backquote,
        egui::Key::Period => Period,
        egui::Key::Semicolon => Semicolon,
        egui::Key::Quote => Quote,
        egui::Key::Exclamationmark => Digit1,
        egui::Key::OpenCurlyBracket => BracketLeft,
        egui::Key::CloseCurlyBracket => BracketRight,
    }
}

pub fn to_livesplit_keycode_alternative(key: &::egui::Key) -> Option<livesplit_hotkey::KeyCode> {
    use livesplit_hotkey::KeyCode::*;

    match key {
        egui::Key::Num0 => Some(Digit0),
        egui::Key::Num1 => Some(Digit1),
        egui::Key::Num2 => Some(Digit2),
        egui::Key::Num3 => Some(Digit3),
        egui::Key::Num4 => Some(Digit4),
        egui::Key::Num5 => Some(Digit5),
        egui::Key::Num6 => Some(Digit6),
        egui::Key::Num7 => Some(Digit7),
        egui::Key::Num8 => Some(Digit8),
        egui::Key::Num9 => Some(Digit9),
        _ => None,
    }
}

pub fn to_livesplit_modifiers(modifiers: &::egui::Modifiers) -> livesplit_hotkey::Modifiers {
    use livesplit_hotkey::Modifiers;
    let mut mods = Modifiers::empty();
    if modifiers.shift {
        mods.insert(Modifiers::SHIFT)
    };
    if modifiers.ctrl {
        mods.insert(Modifiers::CONTROL)
    };
    if modifiers.alt {
        mods.insert(Modifiers::ALT)
    };
    if modifiers.mac_cmd || modifiers.command {
        mods.insert(Modifiers::META)
    };
    mods
}
