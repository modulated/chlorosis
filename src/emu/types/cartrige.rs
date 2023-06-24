use std::mem::transmute;

use super::Byte;

#[derive(Debug)]
struct CartrigeHeaderRaw {
    title: [u8; 16],            // 0x134 - 0x143
    manufacturer_code: [u8; 4], // 0x13F - 0x142
    cgb_flag: u8,               // 0x143 (0x80: backwards compatible, 0xC0: CBG only)
    licensee_code: (u8, u8),    // 0x144 - 145
    sgb_flag: u8,               // 0x146
    mbc_type: u8,               // 0x147 - Memory Bank Controller
    rom_size: u8,               // 0x148 - 32KB < N
    ram_size: u8,               // 0x149
    destination: u8,            // 0x14A
    old_licensee_code: u8,      // 0x14B
    version_number: u8,         // 0x14C
    header_checksum: u8,        // 0x14D - verified at boot
    global_checksum: (u8, u8),  // 0x14E - 0x14F - not verified
}

#[derive(Debug)]
pub struct CartrigeHeader {
    title: String,
    cgb_flag: ColorMode,
    licensee_code: LicenseeCode,
    sgb_flag: SgbSupport,
    mbc_type: MemoryBankControllerType,
    rom_size: u64,
    rom_banks: u16,
    ram_size: u32,
    destination: Destination,
    old_licensee_code: OldLicenseeCode,
    version_number: u8,
    header_checksum: u8,
    global_checksum: u16,
}

impl CartrigeHeader {
    pub fn from_bytes(slice: &[Byte]) -> Self {
        CartrigeHeaderRaw::from_bytes(slice).try_into().unwrap()
    }
}

impl CartrigeHeaderRaw {
    fn from_bytes(slice: &[Byte]) -> Self {
        Self {
            title: slice
                .iter()
                .skip(0x33)
                .take(16)
                .map(|x| x.0)
                .collect::<Vec<u8>>()
                .try_into()
                .expect("Array overflowed"),
            manufacturer_code: slice
                .iter()
                .skip(0x3E)
                .take(4)
                .map(|x| x.0)
                .collect::<Vec<u8>>()
                .try_into()
                .expect("Array overflowed"),
            cgb_flag: slice[0x43].0,
            licensee_code: (slice[0x44].0, slice[0x45].0),
            sgb_flag: slice[0x46].0,
            mbc_type: slice[0x47].0,
            rom_size: slice[0x48].0,
            ram_size: slice[0x49].0,
            destination: slice[0x4A].0,
            old_licensee_code: slice[0x4B].0,
            version_number: slice[0x4C].0,
            header_checksum: slice[0x4D].0,
            global_checksum: (slice[0x4E].0, slice[0x4F].0),
        }
    }
}

impl TryFrom<CartrigeHeaderRaw> for CartrigeHeader {
    type Error = ();
    fn try_from(value: CartrigeHeaderRaw) -> Result<Self, ()> {
        Ok(Self {
            title: String::from_utf8_lossy(&value.title).into_owned(),
            cgb_flag: value.cgb_flag.try_into()?,
            licensee_code: {
                let ascii = (10 * ((value.licensee_code.0 ^ 0x30) as u16)
                    + (value.licensee_code.1 ^ 0x30) as u16) as u8;
                unsafe { transmute(ascii) }
            },
            sgb_flag: value.sgb_flag.try_into()?,
            mbc_type: unsafe { transmute(value.mbc_type) },
            rom_size: get_rom_size(value.rom_size),
            rom_banks: (get_rom_size(value.rom_size) / 0x4000) as u16,
            ram_size: get_ram_size(value.ram_size),
            destination: unsafe { transmute(value.destination) },
            old_licensee_code: unsafe { transmute(value.old_licensee_code) },
            version_number: value.version_number,
            header_checksum: value.header_checksum,
            global_checksum: ((value.global_checksum.0 as u16) << 8)
                + value.global_checksum.1 as u16,
        })
    }
}

#[derive(Debug)]
enum ColorMode {
    BackwardsCompat = 0x80,
    ColorOnly = 0xC0,
}

impl TryFrom<u8> for ColorMode {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, ()> {
        match value {
            0x80 => Ok(Self::BackwardsCompat),
            0xC0 => Ok(Self::ColorOnly),
            _ => panic!("Unknown ColorMode"),
        }
    }
}

#[derive(Debug)]
enum SgbSupport {
    None = 0x00,
    Supported = 0x03,
}

impl TryFrom<u8> for SgbSupport {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, ()> {
        match value {
            0x00 => Ok(Self::None),
            0x03 => Ok(Self::Supported),
            _ => panic!("Unknown SgbSupport"),
        }
    }
}

#[derive(Debug)]
enum Destination {
    Japan = 0x00,
    NotJapan = 0x01,
}

#[derive(Debug)]
enum LicenseeCode {
    None = 0x00,
    NintendoRD1 = 0x01,
    Capcom = 0x08,
    ElectronicArts = 0x13,
    HudsonSoft = 0x18,
    Bai = 0x19,
    Kss = 0x20,
    Pow = 0x21,
    PCMComplete = 0x24,
    SanX = 0x25,
    KemcoJapan = 0x28,
    Seta = 0x29,
    Viacom,
    Nintendo,
    Bandai,
    OceanAcclaim,
    Konami,
    Hector,
    Taito,
    Hudson,
    Banpresto,
    Ubisoft,
    Atlus,
    Malibu,
    Angel,
    BulletProof, // TODO: incomplete
}

#[derive(Debug)]
enum OldLicenseeCode {
    None = 0x00,
    Nintendo = 0x01,
    // TODO: incomplete
    NewCode = 0x33,
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
enum MemoryBankControllerType {
    ROM_ONLY = 0x00,
    MBC1 = 0x01,
    MBC1_RAM = 0x02,
    MBC1_RAM_BATTERY = 0x03,
    MBC2 = 0x05,
    MBC2_BATTERY = 0x06,
    ROM_RAM = 0x08,
    ROM_RAM_BATTERY = 0x09,
    MMM01 = 0x0B,
    MMM01_RAM = 0x0C,
    MMM01_RAM_BATTERY = 0x0D,
    MBC3_TIMER_BATTERY = 0x0F,
    MBC3_TIMER_RAM_BATTERY = 0x10,
    MBC3 = 0x11,
    MBC3_RAM = 0x12,
    MBC3_RAM_BATTERY = 0x13,
    MBC5 = 0x19,
    MBC5_RAM = 0x1A,
    MBC5_RAM_BATTERY = 0x1B,
    MBC5_RUMBLE = 0x1C,
    MBC5_RUMBLE_RAM = 0x1D,
    MBC5_RUMBLE_RAM_BATTERY = 0x1E,
    MBC6 = 0x20,
    MBC7_SENSOR_RUMBLE_RAM_BATTERY = 0x22,
    POCKET_CAMERA = 0xFC,
    BANDAI_TAMA5 = 0xFD,
    HUC3 = 0xFE,
    HUC1_RAM_BATTERY = 0xFF,
}

const fn get_rom_size(code: u8) -> u64 {
    match code {
        0x00..=0x08 => 0x8000 << (code as u64),
        0x52 => 0x120000,
        0x53 => 0x140000,
        0x54 => 0x180000,
        _ => unreachable!(),
    }
}

const fn get_ram_size(code: u8) -> u32 {
    match code {
        0 => 0x0,     // None
        1 => 0x800,   // 2Kb
        2 => 0x2000,  // 8KB
        3 => 0x8000,  // 32KB
        4 => 0x20000, // 128KB
        5 => 0x10000, // 64KB
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use crate::emu::types::cartrige::get_rom_size;

    #[test]
    fn test_rom_size() {
        assert_eq!(get_rom_size(0x00), 0x8000);
        assert_eq!(get_rom_size(0x01), 0x10000);
        assert_eq!(get_rom_size(0x05), 0x100000);
        assert_eq!(get_rom_size(0x54), 0x180000);
    }
}
