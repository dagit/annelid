use annelid::config::app_config::{
    AppConfig, RendererType, YesOrNo, DEFAULT_FRAME_RATE, DEFAULT_POLLING_RATE,
};
use annelid::hotkey::KeyCode;

#[test]
fn default_config_has_expected_hotkeys() {
    let config = AppConfig::default();
    assert_eq!(config.hot_key_start.unwrap().key, KeyCode::Num1);
    assert_eq!(config.hot_key_reset.unwrap().key, KeyCode::Num3);
    assert_eq!(config.hot_key_undo.unwrap().key, KeyCode::Num8);
    assert_eq!(config.hot_key_skip.unwrap().key, KeyCode::Num2);
    assert_eq!(config.hot_key_pause.unwrap().key, KeyCode::Num5);
    assert_eq!(config.hot_key_comparison_next.unwrap().key, KeyCode::Num6);
    assert_eq!(config.hot_key_comparison_prev.unwrap().key, KeyCode::Num4);
}

#[test]
fn default_config_has_expected_defaults() {
    let config = AppConfig::default();
    assert_eq!(config.frame_rate, Some(DEFAULT_FRAME_RATE));
    assert_eq!(config.polling_rate, Some(DEFAULT_POLLING_RATE));
    assert_eq!(config.use_autosplitter, Some(YesOrNo::Yes));
    assert_eq!(config.global_hotkeys, Some(YesOrNo::Yes));
    assert_eq!(config.renderer, Some(RendererType::Gpu));
    assert_eq!(config.reset_timer_on_game_reset, Some(YesOrNo::No));
    assert_eq!(config.reset_game_on_timer_reset, Some(YesOrNo::No));
}

#[test]
fn config_toml_round_trip() {
    let config = AppConfig::default();
    let serialized = toml::to_string_pretty(&config).expect("should serialize");
    let deserialized: AppConfig = toml::from_str(&serialized).expect("should deserialize");

    // Check all hotkey bindings survive
    assert_eq!(
        config.hot_key_start.unwrap().key,
        deserialized.hot_key_start.unwrap().key
    );
    assert_eq!(
        config.hot_key_reset.unwrap().key,
        deserialized.hot_key_reset.unwrap().key
    );
    assert_eq!(
        config.hot_key_undo.unwrap().key,
        deserialized.hot_key_undo.unwrap().key
    );
    assert_eq!(
        config.hot_key_skip.unwrap().key,
        deserialized.hot_key_skip.unwrap().key
    );
    assert_eq!(
        config.hot_key_pause.unwrap().key,
        deserialized.hot_key_pause.unwrap().key
    );

    // Check non-hotkey fields
    assert_eq!(config.frame_rate, deserialized.frame_rate);
    assert_eq!(config.polling_rate, deserialized.polling_rate);
    assert_eq!(config.use_autosplitter, deserialized.use_autosplitter);
    assert_eq!(config.global_hotkeys, deserialized.global_hotkeys);
    assert_eq!(config.renderer, deserialized.renderer);
}

#[test]
fn config_with_no_optional_fields() {
    // Simulate a minimal config file with only required-ish fields
    let toml_str = "";
    let config: AppConfig = toml::from_str(toml_str).expect("should deserialize empty config");
    // All Option fields should be None
    assert!(config.recent_splits.is_none());
    assert!(config.recent_layout.is_none());
    assert!(config.hot_key_start.is_none());
}

#[test]
fn config_partial_fields() {
    let toml_str = r#"
frame_rate = 60.0
use_autosplitter = "Yes"
"#;
    let config: AppConfig = toml::from_str(toml_str).expect("should deserialize partial config");
    assert_eq!(config.frame_rate, Some(60.0));
    assert_eq!(config.use_autosplitter, Some(YesOrNo::Yes));
    assert!(config.hot_key_start.is_none());
}
