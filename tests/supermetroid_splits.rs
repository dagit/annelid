use annelid::autosplitters::supermetroid::{split, SNESState, Settings};

// --- Helpers ---

/// Look up the SNES memory address for a named variable.
fn addr(snes: &SNESState, name: &str) -> usize {
    snes[name].address as usize
}

fn make_snes() -> SNESState {
    let mut snes = SNESState::new();
    snes.update(); // clear do_extra_update
    snes
}

fn write_byte(snes: &mut SNESState, a: usize, val: u8) {
    snes.data[a] = val;
}

fn write_word_le(snes: &mut SNESState, a: usize, val: u16) {
    snes.data[a] = (val & 0xFF) as u8;
    snes.data[a + 1] = (val >> 8) as u8;
}

fn settings_with(keys: &[&str]) -> Settings {
    let mut s = Settings::new();
    for key in keys {
        s.set(key, true);
    }
    s
}

// Room IDs (from the lazy_static roomIDEnum — these are game constants, not our addresses)
const ROOM_WEST_OCEAN: u16 = 0x93FE;
const ROOM_KRAID: u16 = 0xA59F;
const ROOM_VARIA: u16 = 0xA6E2;
const ROOM_BIG_PINK: u16 = 0x9D19;
const ROOM_MOTHER_BRAIN: u16 = 0xDD58;
const ROOM_LANDING_SITE: u16 = 0x91F8;

// --- Negative: all settings disabled ---

#[test]
fn no_split_when_all_settings_disabled() {
    let settings = Settings::new();
    let mut snes = make_snes();
    let a = addr(&snes, "maxMissiles");
    write_byte(&mut snes, a, 5);
    snes.update();
    assert!(!split(&settings, &mut snes));
}

// --- Item count change: firstMissile ---

#[test]
fn split_first_missile() {
    let settings = settings_with(&["ammoPickups", "firstMissile"]);
    let mut snes = make_snes();
    let a = addr(&snes, "maxMissiles");

    write_byte(&mut snes, a, 0);
    snes.update();
    write_byte(&mut snes, a, 5);
    snes.update();
    assert!(split(&settings, &mut snes));
}

#[test]
fn no_split_first_missile_wrong_count() {
    let settings = settings_with(&["ammoPickups", "firstMissile"]);
    let mut snes = make_snes();
    let a = addr(&snes, "maxMissiles");

    write_byte(&mut snes, a, 5);
    snes.update();
    write_byte(&mut snes, a, 10);
    snes.update();
    assert!(!split(&settings, &mut snes));
}

// --- Room + item bits: oceanBottomMissiles ---

#[test]
fn split_ocean_bottom_missiles() {
    let settings = settings_with(&[
        "ammoPickups",
        "specificMissiles",
        "crateriaMissiles",
        "oceanBottomMissiles",
    ]);
    let mut snes = make_snes();
    let room = addr(&snes, "roomID");
    let items = addr(&snes, "crateriaItems");

    write_word_le(&mut snes, room, ROOM_WEST_OCEAN);
    write_byte(&mut snes, items, 0);
    snes.update();
    write_byte(&mut snes, items, 2);
    snes.update();

    assert!(split(&settings, &mut snes));
}

#[test]
fn no_split_ocean_bottom_missiles_wrong_room() {
    let settings = settings_with(&[
        "ammoPickups",
        "specificMissiles",
        "crateriaMissiles",
        "oceanBottomMissiles",
    ]);
    let mut snes = make_snes();
    let room = addr(&snes, "roomID");
    let items = addr(&snes, "crateriaItems");

    write_word_le(&mut snes, room, ROOM_LANDING_SITE);
    write_byte(&mut snes, items, 0);
    snes.update();
    write_byte(&mut snes, items, 2);
    snes.update();

    assert!(!split(&settings, &mut snes));
}

// --- Boss flag transition: kraid ---

#[test]
fn split_kraid() {
    let settings = settings_with(&["bosses", "kraid"]);
    let mut snes = make_snes();
    let room = addr(&snes, "roomID");
    let bosses = addr(&snes, "brinstarBosses");

    write_word_le(&mut snes, room, ROOM_KRAID);
    write_byte(&mut snes, bosses, 0);
    snes.update();
    write_byte(&mut snes, bosses, 1);
    snes.update();

    assert!(split(&settings, &mut snes));
}

#[test]
fn no_split_kraid_wrong_room() {
    let settings = settings_with(&["bosses", "kraid"]);
    let mut snes = make_snes();
    let room = addr(&snes, "roomID");
    let bosses = addr(&snes, "brinstarBosses");

    write_word_le(&mut snes, room, ROOM_LANDING_SITE);
    write_byte(&mut snes, bosses, 0);
    snes.update();
    write_byte(&mut snes, bosses, 1);
    snes.update();

    assert!(!split(&settings, &mut snes));
}

// --- Equipment pickup: varia suit ---

#[test]
fn split_varia_suit() {
    let settings = settings_with(&["majorUpgrades", "variaSuit"]);
    let mut snes = make_snes();
    let room = addr(&snes, "roomID");
    let equips = addr(&snes, "unlockedEquips2");

    write_word_le(&mut snes, room, ROOM_VARIA);
    write_byte(&mut snes, equips, 0);
    snes.update();
    write_byte(&mut snes, equips, 1);
    snes.update();

    assert!(split(&settings, &mut snes));
}

// --- Beam pickup: charge beam ---

#[test]
fn split_charge_beam() {
    let settings = settings_with(&["beamUpgrades", "chargeBeam"]);
    let mut snes = make_snes();
    let room = addr(&snes, "roomID");
    let charge = addr(&snes, "unlockedCharge");

    write_word_le(&mut snes, room, ROOM_BIG_PINK);
    write_byte(&mut snes, charge, 0);
    snes.update();
    write_byte(&mut snes, charge, 0x10);
    snes.update();

    assert!(split(&settings, &mut snes));
}

// --- MB phase: mb1 ---

#[test]
fn split_mb1() {
    let settings = settings_with(&["bosses", "mb1"]);
    let mut snes = make_snes();
    let room = addr(&snes, "roomID");
    let gs = addr(&snes, "gameState");
    let mbhp = addr(&snes, "motherBrainHP");

    write_word_le(&mut snes, room, ROOM_MOTHER_BRAIN);
    write_byte(&mut snes, gs, 0x08); // normalGameplay
    write_word_le(&mut snes, mbhp, 0);
    snes.update();
    write_word_le(&mut snes, mbhp, 0x4650); // phase2 max HP
    snes.update();

    assert!(split(&settings, &mut snes));
}

// --- Escape (RTA finish): shipAI transition ---

#[test]
fn split_rta_finish() {
    let settings = settings_with(&["rtaFinish"]);
    let mut snes = make_snes();
    let ef = addr(&snes, "eventFlags");
    let ship = addr(&snes, "shipAI");

    write_byte(&mut snes, ef, 0x40); // zebesAblaze
    write_word_le(&mut snes, ship, 0x0000);
    snes.update();
    write_word_le(&mut snes, ship, 0xaa4f);
    snes.update();

    assert!(split(&settings, &mut snes));
}

#[test]
fn no_split_rta_finish_without_zebes_ablaze() {
    let settings = settings_with(&["rtaFinish"]);
    let mut snes = make_snes();
    let ef = addr(&snes, "eventFlags");
    let ship = addr(&snes, "shipAI");

    write_byte(&mut snes, ef, 0);
    write_word_le(&mut snes, ship, 0x0000);
    snes.update();
    write_word_le(&mut snes, ship, 0xaa4f);
    snes.update();

    assert!(!split(&settings, &mut snes));
}

// --- Setting enabled but no memory change ---

#[test]
fn no_split_kraid_enabled_no_memory_change() {
    let settings = settings_with(&["bosses", "kraid"]);
    let mut snes = make_snes();
    let room = addr(&snes, "roomID");

    write_word_le(&mut snes, room, ROOM_KRAID);
    snes.update();
    snes.update();

    assert!(!split(&settings, &mut snes));
}
