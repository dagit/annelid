use annelid::hotkey::{HotKey, KeyCode, KeyEvent, Modifiers};
use proptest::prelude::*;
use strum::IntoEnumIterator;

fn arb_keycode() -> impl Strategy<Value = KeyCode> {
    let variants: Vec<KeyCode> = KeyCode::iter().collect();
    (0..variants.len()).prop_map(move |i| variants[i])
}

fn arb_modifiers() -> impl Strategy<Value = Modifiers> {
    (
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(|(alt, ctrl, shift, mac_cmd, command)| Modifiers {
            alt,
            ctrl,
            shift,
            mac_cmd,
            command,
        })
}

proptest! {
    #[test]
    fn hotkey_toml_round_trip(key in arb_keycode(), mods in arb_modifiers()) {
        let hk = HotKey { key, modifiers: mods };
        let s = toml::to_string_pretty(&hk).expect("should serialize");
        let hk2: HotKey = toml::from_str(&s).expect("should deserialize");
        prop_assert_eq!(hk.key, hk2.key);
        prop_assert_eq!(hk.modifiers, hk2.modifiers);
    }

    #[test]
    fn hotkey_json_round_trip(key in arb_keycode(), mods in arb_modifiers()) {
        let hk = HotKey { key, modifiers: mods };
        let s = serde_json::to_string(&hk).expect("should serialize");
        let hk2: HotKey = serde_json::from_str(&s).expect("should deserialize");
        prop_assert_eq!(hk.key, hk2.key);
        prop_assert_eq!(hk.modifiers, hk2.modifiers);
    }

    #[test]
    fn keycode_toml_round_trip(key in arb_keycode()) {
        #[derive(serde_derive::Serialize, serde_derive::Deserialize)]
        struct Wrapper { key: KeyCode }
        let w = Wrapper { key };
        let s = toml::to_string_pretty(&w).expect("should serialize");
        let w2: Wrapper = toml::from_str(&s).expect("should deserialize");
        prop_assert_eq!(w.key, w2.key);
    }
}

#[test]
fn all_keycodes_serialize_round_trip() {
    for key in KeyCode::iter() {
        #[derive(serde_derive::Serialize, serde_derive::Deserialize)]
        struct W {
            key: KeyCode,
        }
        let w = W { key };
        let s = toml::to_string_pretty(&w).unwrap_or_else(|e| panic!("serialize {key:?}: {e}"));
        let w2: W = toml::from_str(&s).unwrap_or_else(|e| panic!("deserialize {key:?}: {e}"));
        assert_eq!(w.key, w2.key, "round-trip failed for {key:?}");
    }
}

#[test]
fn default_modifiers_all_false() {
    let m = Modifiers::default();
    assert!(!m.alt);
    assert!(!m.ctrl);
    assert!(!m.shift);
    assert!(!m.mac_cmd);
    assert!(!m.command);
}

#[test]
fn existing_config_format_compatible() {
    let toml_str = r#"
key = "Num1"
[modifiers]
alt = false
ctrl = false
shift = false
mac_cmd = false
command = false
"#;
    let hk: HotKey = toml::from_str(toml_str).expect("should deserialize existing format");
    assert_eq!(hk.key, KeyCode::Num1);
    assert_eq!(hk.modifiers, Modifiers::default());
}

// --- capture_from_event tests ---

#[test]
fn capture_ignores_release_events() {
    let event = KeyEvent {
        key: KeyCode::A,
        modifiers: Modifiers::default(),
        pressed: false,
    };
    assert!(HotKey::capture_from_event(&event).is_none());
}

#[test]
fn capture_escape_cancels() {
    let event = KeyEvent {
        key: KeyCode::Escape,
        modifiers: Modifiers::default(),
        pressed: true,
    };
    let result = HotKey::capture_from_event(&event);
    assert_eq!(result, Some(None));
}

#[test]
fn capture_normal_key() {
    let event = KeyEvent {
        key: KeyCode::F5,
        modifiers: Modifiers::default(),
        pressed: true,
    };
    let result = HotKey::capture_from_event(&event);
    let hotkey = result.unwrap().unwrap();
    assert_eq!(hotkey.key, KeyCode::F5);
    assert_eq!(hotkey.modifiers, Modifiers::default());
}

#[test]
fn capture_preserves_modifiers() {
    let mods = Modifiers {
        alt: false,
        ctrl: true,
        shift: true,
        mac_cmd: false,
        command: false,
    };
    let event = KeyEvent {
        key: KeyCode::S,
        modifiers: mods,
        pressed: true,
    };
    let hotkey = HotKey::capture_from_event(&event).unwrap().unwrap();
    assert_eq!(hotkey.key, KeyCode::S);
    assert!(hotkey.modifiers.ctrl);
    assert!(hotkey.modifiers.shift);
    assert!(!hotkey.modifiers.alt);
}

#[test]
fn capture_escape_with_modifiers_still_cancels() {
    let mods = Modifiers {
        alt: false,
        ctrl: true,
        shift: false,
        mac_cmd: false,
        command: false,
    };
    let event = KeyEvent {
        key: KeyCode::Escape,
        modifiers: mods,
        pressed: true,
    };
    assert_eq!(HotKey::capture_from_event(&event), Some(None));
}

proptest! {
    #[test]
    fn capture_any_non_escape_key_succeeds(key in arb_keycode(), mods in arb_modifiers()) {
        let event = KeyEvent { key, modifiers: mods, pressed: true };
        let result = HotKey::capture_from_event(&event);
        match key {
            KeyCode::Escape => prop_assert_eq!(result, Some(None)),
            _ => {
                let hk = result.unwrap().unwrap();
                prop_assert_eq!(hk.key, key);
                prop_assert_eq!(hk.modifiers, mods);
            }
        }
    }

    #[test]
    fn capture_release_always_ignored(key in arb_keycode(), mods in arb_modifiers()) {
        let event = KeyEvent { key, modifiers: mods, pressed: false };
        prop_assert!(HotKey::capture_from_event(&event).is_none());
    }
}
