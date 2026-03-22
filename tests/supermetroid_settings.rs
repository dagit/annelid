use annelid::autosplitters::supermetroid::Settings;

#[test]
fn default_settings_have_roots() {
    let s = Settings::new();
    let roots = s.roots();
    assert!(!roots.is_empty());
    // ammoPickups is a known root
    assert!(roots.contains(&"ammoPickups".to_string()));
}

#[test]
fn get_disabled_returns_false() {
    let s = Settings::new();
    // firstMissile defaults to false
    assert!(!s.get("firstMissile"));
}

#[test]
fn get_enabled_with_parent_disabled_returns_false() {
    let mut s = Settings::new();
    // Enable a child but ensure its parent chain is disabled
    // "oceanBottomMissiles" -> "crateriaMissiles" -> "specificMissiles" -> "ammoPickups"
    s.set("oceanBottomMissiles", true);
    // specificMissiles defaults to false, so the parent chain blocks it
    assert!(!s.get("oceanBottomMissiles"));
}

#[test]
fn get_enabled_with_full_parent_chain_returns_true() {
    let mut s = Settings::new();
    // Enable the full chain: ammoPickups -> specificMissiles -> crateriaMissiles -> oceanBottomMissiles
    s.set("ammoPickups", true);
    s.set("specificMissiles", true);
    s.set("crateriaMissiles", true);
    s.set("oceanBottomMissiles", true);
    assert!(s.get("oceanBottomMissiles"));
}

#[test]
fn set_and_get_round_trip() {
    let mut s = Settings::new();
    // ammoPickups is a root, no parent
    s.set("ammoPickups", false);
    assert!(!s.get("ammoPickups"));
    s.set("ammoPickups", true);
    assert!(s.get("ammoPickups"));
}

#[test]
fn get_unknown_key_returns_false() {
    let s = Settings::new();
    assert!(!s.get("thisKeyDoesNotExist"));
}

#[test]
fn roots_do_not_include_children() {
    let s = Settings::new();
    let roots = s.roots();
    // firstMissile has a parent, should not be a root
    assert!(!roots.contains(&"firstMissile".to_string()));
}

#[test]
fn split_on_anypercent_enables_expected_keys() {
    let mut s = Settings::new();
    s.split_on_anypercent();
    // kraid should be enabled in any% preset
    assert!(s.get("kraid"));
}

#[test]
fn settings_serde_round_trip() {
    let s = Settings::new();
    let json = serde_json::to_string(&s).expect("serialize");
    let s2: Settings = serde_json::from_str(&json).expect("deserialize");
    // Spot check a few keys
    assert_eq!(s.get("ammoPickups"), s2.get("ammoPickups"));
    assert_eq!(s.get("firstMissile"), s2.get("firstMissile"));
}
