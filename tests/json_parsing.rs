use annelid::autosplitters::json::Splits;

const MINIMAL_SPLITS: &str = r#"{
    "game": "Test Game",
    "definitions": [
        {
            "name": "first_split",
            "address": "0x1234",
            "value": "0x01",
            "type": "bit"
        }
    ]
}"#;

const FULL_SPLITS: &str = r#"{
    "game": "Test Game",
    "autostart": {
        "active": "1",
        "note": "Start on game begin",
        "address": "0x0998",
        "value": "0x1F",
        "type": "eq"
    },
    "definitions": [
        {
            "name": "boss_kill",
            "address": "0xD829",
            "value": "0x01",
            "type": "bit",
            "note": "Kraid defeated",
            "more": [
                {
                    "address": "0x079B",
                    "value": "0xA59F",
                    "type": "weq"
                }
            ]
        },
        {
            "name": "item_pickup",
            "address": "0x09C8",
            "value": "0x05",
            "type": "eq",
            "next": [
                {
                    "address": "0x09C8",
                    "value": "0x0A",
                    "type": "eq"
                }
            ]
        }
    ]
}"#;

#[test]
fn parse_minimal() {
    let splits = Splits::parse(MINIMAL_SPLITS).expect("should parse");
    assert_eq!(splits.game, "Test Game");
    assert_eq!(splits.definitions.len(), 1);
    assert_eq!(splits.definitions[0].name, "first_split");
    assert_eq!(splits.definitions[0].check.address, 0x1234);
    assert_eq!(splits.definitions[0].check.value, 0x01);
    assert!(splits.autostart.is_none());
    assert!(splits.definitions[0].more.is_none());
    assert!(splits.definitions[0].next.is_none());
}

#[test]
fn parse_full_with_autostart() {
    let splits = Splits::parse(FULL_SPLITS).expect("should parse");
    assert_eq!(splits.definitions.len(), 2);

    // Autostart
    let autostart = splits.autostart.expect("should have autostart");
    match autostart {
        annelid::autosplitters::json::Autostart::Active { check } => {
            assert_eq!(check.address, 0x0998);
            assert_eq!(check.value, 0x1F);
        }
        _ => panic!("expected active autostart"),
    }
}

#[test]
fn parse_more_and_next_fields() {
    let splits = Splits::parse(FULL_SPLITS).expect("should parse");

    // First definition has "more"
    let more = splits.definitions[0]
        .more
        .as_ref()
        .expect("should have more");
    assert_eq!(more.len(), 1);
    assert_eq!(more[0].address, 0x079B);

    // Second definition has "next"
    let next = splits.definitions[1]
        .next
        .as_ref()
        .expect("should have next");
    assert_eq!(next.len(), 1);
    assert_eq!(next[0].value, 0x0A);
}

#[test]
fn hex_parsing_with_prefix() {
    // Addresses use "0x" prefix
    let splits = Splits::parse(MINIMAL_SPLITS).expect("should parse");
    assert_eq!(splits.definitions[0].check.address, 0x1234);
}

#[test]
fn hex_parsing_without_prefix() {
    let json = r#"{
        "game": "Test",
        "definitions": [{
            "name": "no_prefix",
            "address": "BEEF",
            "value": "42",
            "type": "eq"
        }]
    }"#;
    let splits = Splits::parse(json).expect("should parse hex without 0x prefix");
    assert_eq!(splits.definitions[0].check.address, 0xBEEF);
    assert_eq!(splits.definitions[0].check.value, 0x42);
}

#[test]
fn inactive_autostart() {
    let json = r#"{
        "game": "Test",
        "autostart": {
            "active": "0",
            "note": "Disabled"
        },
        "definitions": []
    }"#;
    let splits = Splits::parse(json).expect("should parse");
    match splits.autostart.expect("should have autostart") {
        annelid::autosplitters::json::Autostart::Inactive { note } => {
            assert_eq!(note, Some("Disabled".to_string()));
        }
        _ => panic!("expected inactive autostart"),
    }
}

#[test]
fn all_check_types_parse() {
    let types = [
        "bit", "eq", "gt", "lt", "gte", "lte", "wbit", "weq", "wgt", "wlt", "wgte", "wlte",
    ];
    for typ in types {
        let json = format!(
            r#"{{"game":"T","definitions":[{{"name":"x","address":"0x00","value":"0x00","type":"{typ}"}}]}}"#
        );
        Splits::parse(&json).unwrap_or_else(|e| panic!("failed to parse type {typ}: {e}"));
    }
}

#[test]
fn game_alias_name() {
    // "name" should also work as alias for "game"
    let json = r#"{"name": "My Game", "definitions": []}"#;
    let splits = Splits::parse(json).expect("should parse");
    assert_eq!(splits.game, "My Game");
}

#[test]
fn malformed_json_returns_error() {
    assert!(Splits::parse("not json at all").is_err());
}

#[test]
fn missing_required_fields_returns_error() {
    // Missing "definitions"
    let json = r#"{"game": "Test"}"#;
    assert!(Splits::parse(json).is_err());
}

#[test]
fn invalid_hex_returns_error() {
    let json = r#"{
        "game": "Test",
        "definitions": [{
            "name": "bad",
            "address": "0xZZZZ",
            "value": "0x00",
            "type": "eq"
        }]
    }"#;
    assert!(Splits::parse(json).is_err());
}
