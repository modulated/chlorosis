//! Headless test-ROM harness.
//!
//! Game Boy test ROMs (Blargg's suite especially) report results through the
//! serial port: they print text a byte at a time and finish with "Passed" or
//! "Failed". These tests drive a `Device` directly through its public `tick`,
//! with no window or real-time pacing, and read `serial_output` back.
//!
//! `serial_console_reports_output` is fully self-contained - it assembles a
//! tiny ROM by hand. `blargg_test_rom` runs a real Blargg ROM when one is
//! pointed to by `CHLOROSIS_TEST_ROM`, and otherwise skips.

use std::io::Write;

use chlorosis_core::{Device, TICKS_PER_FRAME};

/// Wrap a program in a 32 KB ROM-only cartridge image, placed at the 0x0100
/// entry point, with a header just valid enough to parse (ROM ONLY, 32 KB).
fn cartridge_with(program: &[u8]) -> tempfile::NamedTempFile {
    let mut rom = vec![0u8; 0x8000];
    rom[0x0100..0x0100 + program.len()].copy_from_slice(program);
    rom[0x0147] = 0x00; // MBC: ROM ONLY
    rom[0x0148] = 0x00; // ROM size: 32 KB

    let mut file = tempfile::NamedTempFile::new().expect("temp rom");
    file.write_all(&rom).expect("write rom");
    file.flush().expect("flush rom");
    file
}

#[test]
fn serial_console_reports_output() {
    // Print "OK" over the serial port, then spin. For each byte:
    //   3E xx   LD A, byte
    //   E0 01   LDH (0xFF01), A   ; stage it in SB
    //   3E 81   LD A, 0x81
    //   E0 02   LDH (0xFF02), A   ; start transfer -> byte is shifted out
    let mut program = vec![];
    for byte in *b"OK" {
        program.extend_from_slice(&[0x3E, byte, 0xE0, 0x01, 0x3E, 0x81, 0xE0, 0x02]);
    }
    program.extend_from_slice(&[0x18, 0xFE]); // JR -2: spin in place

    let rom = cartridge_with(&program);
    let mut dev = Device::new();
    dev.load_cartrige(rom.path()).expect("load");

    // A single frame is far more than enough for a handful of instructions.
    dev.tick(TICKS_PER_FRAME);

    assert_eq!(dev.serial_output(), b"OK");
}

#[test]
fn blargg_test_rom() {
    let Ok(path) = std::env::var("CHLOROSIS_TEST_ROM") else {
        eprintln!("skipping: set CHLOROSIS_TEST_ROM to a Blargg .gb/.gbc to run this");
        return;
    };

    let mut dev = Device::new();
    dev.load_cartrige(&path).expect("load test rom");

    // Blargg ROMs finish within a few seconds of emulated time; cap the run so a
    // hang fails instead of looping forever (~30 s at 59.7 fps).
    for _ in 0..1800 {
        dev.tick(TICKS_PER_FRAME);
        let out = String::from_utf8_lossy(dev.serial_output()).into_owned();
        if out.contains("Passed") || out.contains("Failed") {
            assert!(out.contains("Passed"), "test ROM reported:\n{out}");
            return;
        }
    }

    panic!(
        "test ROM did not finish; serial output so far:\n{}",
        String::from_utf8_lossy(dev.serial_output())
    );
}
