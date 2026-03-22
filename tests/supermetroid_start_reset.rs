use annelid::autosplitters::supermetroid::{MemoryWatcher, SNESState, Width};

// --- Helpers ---

/// Look up the SNES memory address for a named variable.
fn addr(snes: &SNESState, name: &str) -> usize {
    snes[name].address as usize
}

fn make_snes() -> SNESState {
    let mut snes = SNESState::new();
    // Double-update to clear the do_extra_update flag and initialize old=current=0
    snes.update();
    snes
}

fn write_byte(snes: &mut SNESState, a: usize, val: u8) {
    snes.data[a] = val;
}

fn write_word_le(snes: &mut SNESState, a: usize, val: u16) {
    snes.data[a] = (val & 0xFF) as u8;
    snes.data[a + 1] = (val >> 8) as u8;
}

// --- MemoryWatcher ---

#[test]
fn memory_watcher_byte_update() {
    let mut mem = vec![0u8; 256];
    let mut w = MemoryWatcher::new(0x10, Width::Byte);

    mem[0x10] = 42;
    w.update_value(&mem);
    assert_eq!(w.current, 42);
    assert_eq!(w.old, 0);

    mem[0x10] = 99;
    w.update_value(&mem);
    assert_eq!(w.current, 99);
    assert_eq!(w.old, 42);
}

#[test]
fn memory_watcher_word_little_endian() {
    let mut mem = vec![0u8; 256];
    let mut w = MemoryWatcher::new(0x20, Width::Word);

    // 0x1234 in little-endian: low byte first
    mem[0x20] = 0x34;
    mem[0x21] = 0x12;
    w.update_value(&mem);
    assert_eq!(w.current, 0x1234);
    assert_eq!(w.old, 0);
}

#[test]
fn memory_watcher_tracks_old_current() {
    let mut mem = vec![0u8; 256];
    let mut w = MemoryWatcher::new(0x00, Width::Byte);

    mem[0x00] = 1;
    w.update_value(&mem);
    mem[0x00] = 2;
    w.update_value(&mem);
    mem[0x00] = 3;
    w.update_value(&mem);

    assert_eq!(w.old, 2);
    assert_eq!(w.current, 3);
}

// --- start() ---

#[test]
fn start_normal() {
    let mut snes = make_snes();
    let gs = addr(&snes, "gameState");
    // gameState transitions from 2 -> 0x1f
    write_byte(&mut snes, gs, 2);
    snes.update();
    write_byte(&mut snes, gs, 0x1f);
    snes.update();
    assert!(snes.start());
}

#[test]
fn start_cutscene() {
    let mut snes = make_snes();
    let gs = addr(&snes, "gameState");
    // gameState transitions from 0x1E -> 0x1F
    write_byte(&mut snes, gs, 0x1E);
    snes.update();
    write_byte(&mut snes, gs, 0x1F);
    snes.update();
    assert!(snes.start());
}

#[test]
fn start_zebes() {
    let mut snes = make_snes();
    let gs = addr(&snes, "gameState");
    // gameState transitions from 5 -> 6
    write_byte(&mut snes, gs, 5);
    snes.update();
    write_byte(&mut snes, gs, 6);
    snes.update();
    assert!(snes.start());
}

#[test]
fn no_start_when_state_unchanged() {
    let mut snes = make_snes();
    let gs = addr(&snes, "gameState");
    write_byte(&mut snes, gs, 0x1f);
    snes.update();
    // No transition — same value
    snes.update();
    assert!(!snes.start());
}

#[test]
fn no_start_wrong_transition() {
    let mut snes = make_snes();
    let gs = addr(&snes, "gameState");
    write_byte(&mut snes, gs, 0x08);
    snes.update();
    write_byte(&mut snes, gs, 0x0B);
    snes.update();
    assert!(!snes.start());
}

// --- reset() ---

#[test]
fn reset_when_room_id_goes_to_zero() {
    let mut snes = make_snes();
    let room = addr(&snes, "roomID");
    write_word_le(&mut snes, room, 0x91F8); // landingSite
    snes.update();
    write_word_le(&mut snes, room, 0);
    snes.update();
    assert!(snes.reset());
}

#[test]
fn no_reset_room_stays_nonzero() {
    let mut snes = make_snes();
    let room = addr(&snes, "roomID");
    write_word_le(&mut snes, room, 0x91F8);
    snes.update();
    write_word_le(&mut snes, room, 0x93AA);
    snes.update();
    assert!(!snes.reset());
}

#[test]
fn no_reset_room_stays_zero() {
    let mut snes = make_snes();
    // roomID is 0 (default), update twice
    snes.update();
    snes.update();
    assert!(!snes.reset());
}

// --- gametime_to_seconds() ---

#[test]
fn gametime_zero() {
    let mut snes = make_snes();
    snes.update();
    let ts = snes.gametime_to_seconds();
    assert!((ts.total_seconds() - 0.0).abs() < 0.001);
}

#[test]
fn gametime_known_value() {
    let mut snes = make_snes();
    let hours = addr(&snes, "igtHours");
    let minutes = addr(&snes, "igtMinutes");
    let seconds = addr(&snes, "igtSeconds");
    write_byte(&mut snes, hours, 1); // 1 hour
    write_byte(&mut snes, minutes, 30); // 30 minutes
    write_byte(&mut snes, seconds, 45); // 45 seconds
    snes.update();
    let ts = snes.gametime_to_seconds();
    let expected = 1.0 * 3600.0 + 30.0 * 60.0 + 45.0;
    assert!(
        (ts.total_seconds() - expected).abs() < 0.001,
        "Expected {expected}, got {}",
        ts.total_seconds()
    );
}
