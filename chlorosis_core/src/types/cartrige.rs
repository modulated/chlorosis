use std::{fmt::Display, mem::transmute};

use crate::Byte;
#[allow(dead_code)]
#[derive(Debug)]
struct CartrigeHeaderRaw {
    title: [u8; 16],             // 0x134 - 0x143
    manufacturer_code: [u8; 4],  // 0x13F - 0x142
    cgb_flag: u8,                // 0x143 (0x80: backwards compatible, 0xC0: CBG only)
    new_licensee_code: (u8, u8), // 0x144 - 145
    sgb_flag: u8,                // 0x146
    mbc_type: u8,                // 0x147 - Memory Bank Controller
    rom_size: u8,                // 0x148 - 32KB < N
    ram_size: u8,                // 0x149
    destination: u8,             // 0x14A
    old_licensee_code: u8,       // 0x14B
    version_number: u8,          // 0x14C
    header_checksum: u8,         // 0x14D - verified at boot
    global_checksum: (u8, u8),   // 0x14E - 0x14F - not verified
}

// TODO - merge old and new licensee codes
#[allow(dead_code)]
#[derive(Debug)]
pub struct CartrigeHeader {
    title: String,
    cgb_flag: ColorMode,
    licensee_name: String,
    licensee_code: u8,
    sgb_flag: SgbSupport,
    mbc_type: MemoryBankControllerType,
    rom_size: u64,
    rom_banks: u16,
    ram_size: u32,
    destination: Destination,
    version_number: u8,
    header_checksum: u8,
    global_checksum: u16,
}

impl CartrigeHeader {
    pub fn from_bytes(slice: &[Byte]) -> Self {
        CartrigeHeaderRaw::from_bytes(slice).into()
    }
}

// TODO - remove transmute - impl from/to conversion
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
                .expect("Should not overflow"),
            manufacturer_code: slice
                .iter()
                .skip(0x3E)
                .take(4)
                .map(|x| x.0)
                .collect::<Vec<u8>>()
                .try_into()
                .expect("Should not overflow"),
            cgb_flag: slice[0x43].0,
            new_licensee_code: (slice[0x44].0, slice[0x45].0),
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

    const fn get_licensee_code(&self) -> u8 {
        if self.old_licensee_code == 0x33 {
            merge_nibbles(self.new_licensee_code.0, self.new_licensee_code.1)
        } else {
            self.old_licensee_code
        }
    }

    fn get_licensee_name(&self) -> String {
        if self.old_licensee_code == 0x33 {
            NewLicenseeCode::from(merge_nibbles(
                self.new_licensee_code.0,
                self.new_licensee_code.1,
            ))
            .to_string()
        } else {
            OldLicenseeCode::from(self.old_licensee_code).to_string()
        }
    }
}

const fn merge_nibbles(a: u8, b: u8) -> u8 {
    (a << 4) + b
}

impl From<CartrigeHeaderRaw> for CartrigeHeader {
    fn from(value: CartrigeHeaderRaw) -> Self {
        Self {
            title: String::from_utf8_lossy(&value.title).into_owned(),
            cgb_flag: value.cgb_flag.into(),
            licensee_name: value.get_licensee_name(),
            licensee_code: value.get_licensee_code(),
            sgb_flag: value.sgb_flag.into(),
            mbc_type: unsafe { transmute(value.mbc_type) },
            rom_size: get_rom_size(value.rom_size),
            rom_banks: (get_rom_size(value.rom_size) / 0x4000) as u16,
            ram_size: get_ram_size(value.ram_size),
            destination: value.destination.into(),
            version_number: value.version_number,
            header_checksum: value.header_checksum,
            global_checksum: ((value.global_checksum.0 as u16) << 8)
                + value.global_checksum.1 as u16,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum ColorMode {
    BackwardsCompat,
    ColorOnly,
    Unknown,
}

impl From<u8> for ColorMode {
    fn from(value: u8) -> Self {
        match value {
            0x80 => Self::BackwardsCompat,
            0xC0 => Self::ColorOnly,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum SgbSupport {
    None,
    Supported,
    Unknown,
}

impl From<u8> for SgbSupport {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::None,
            0x03 => Self::Supported,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Destination {
    Japan,
    NotJapan,
    Unknown,
}

impl From<u8> for Destination {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::Japan,
            0x01 => Self::NotJapan,
            _ => Self::Unknown,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum NewLicenseeCode {
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
    Viacom = 0x30,
    Nintendo = 0x31,
    Bandai = 0x32,
    OceanAcclaim = 0x33,
    Konami = 0x34,
    Hector = 0x35,
    Taito = 0x37,
    Hudson = 0x38,
    Banpresto = 0x39,
    UbiSoft = 0x41,
    Atlus = 0x42,
    Malibu = 0x44,
    Angel = 0x46,
    BulletProof = 0x47,
    Irem = 0x49,
    Absolute = 0x50,
    Acclaim = 0x51,
    Activision = 0x52,
    AmericanSammy = 0x53,
    // Konami = 0x54,
    HiTechEntertainment = 0x55,
    Ljn = 0x56,
    Matchbox = 0x57,
    Mattel = 0x58,
    MiltonBradley = 0x59,
    Titus = 0x60,
    Virgin = 0x61,
    LucasArts = 0x64,
    Ocean = 0x67,
    // ElectronicArts = 0x69,
    Interplay = 0x71,
    Broderbund = 0x72,
    Sculptured = 0x73,
    Sci = 0x75,
    Thq = 0x78,
    Accolade = 0x79,
    Misawa = 0x80,
    Lozc = 0x83,
    TokumaShotenIntermedia = 0x86,
    TsukudaOriginal = 0x87,
    Chunsoft = 0x91,
    VideoSystem = 0x92,
    // OceanAcclaim = 0x93,
    Varie = 0x95,
    YonezawasPal = 0x96,
    Kaneko = 0x97,
    PackInSoft = 0x99,
    KonamiYuGiOh = 0xA4,
    Unknown,
}

impl From<u8> for NewLicenseeCode {
    fn from(value: u8) -> Self {
        use NewLicenseeCode::*;
        match value {
            0x00 => None,
            0x01 => NintendoRD1,
            0x08 => Capcom,
            0x13 => ElectronicArts,
            0x18 => HudsonSoft,
            0x19 => Bai,
            0x20 => Kss,
            0x21 => Pow,
            0x24 => PCMComplete,
            0x25 => SanX,
            0x28 => KemcoJapan,
            0x29 => Seta,
            0x30 => Viacom,
            0x31 => Nintendo,
            0x32 => Bandai,
            0x33 => OceanAcclaim,
            0x34 => Konami,
            0x35 => Hector,
            0x37 => Taito,
            0x38 => Hudson,
            0x39 => Banpresto,
            0x41 => UbiSoft,
            0x42 => Atlus,
            0x44 => Malibu,
            0x46 => Angel,
            0x47 => BulletProof,
            0x49 => Irem,
            0x50 => Absolute,
            0x51 => Acclaim,
            0x52 => Activision,
            0x53 => AmericanSammy,
            0x54 => Konami,
            0x55 => HiTechEntertainment,
            0x56 => Ljn,
            0x57 => Matchbox,
            0x58 => Mattel,
            0x59 => MiltonBradley,
            0x60 => Titus,
            0x61 => Virgin,
            0x64 => LucasArts,
            0x67 => Ocean,
            0x69 => ElectronicArts,
            0x71 => Interplay,
            0x72 => Broderbund,
            0x73 => Sculptured,
            0x75 => Sci,
            0x78 => Thq,
            0x79 => Accolade,
            0x80 => Misawa,
            0x83 => Lozc,
            0x86 => TokumaShotenIntermedia,
            0x87 => TsukudaOriginal,
            0x91 => Chunsoft,
            0x92 => VideoSystem,
            0x93 => OceanAcclaim,
            0x95 => Varie,
            0x96 => YonezawasPal,
            0x97 => Kaneko,
            0x99 => PackInSoft,
            0xA4 => KonamiYuGiOh,
            _ => Unknown,
        }
    }
}

impl Display for NewLicenseeCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use NewLicenseeCode::*;
        match self {
            None => Ok(write!(f, "None")?),
            NintendoRD1 => Ok(write!(f, "Nintendo R&D1")?),
            Capcom => Ok(write!(f, "Capcom")?),
            ElectronicArts => Ok(write!(f, "Electronic Arts")?),
            HudsonSoft => Ok(write!(f, "Hudson Soft")?),
            Bai => Ok(write!(f, "b-ai")?),
            Kss => Ok(write!(f, "kss")?),
            Pow => Ok(write!(f, "pow")?),
            PCMComplete => Ok(write!(f, "PCM Complete")?),
            SanX => Ok(write!(f, "san-x")?),
            KemcoJapan => Ok(write!(f, "Kemco Japan")?),
            Seta => Ok(write!(f, "seta")?),
            Viacom => Ok(write!(f, "Viacom")?),
            Nintendo => Ok(write!(f, "Nintendo")?),
            Bandai => Ok(write!(f, "Bandai")?),
            OceanAcclaim => Ok(write!(f, "Ocean/Acclaim")?),
            Konami => Ok(write!(f, "Konami")?),
            Hector => Ok(write!(f, "Hector")?),
            Taito => Ok(write!(f, "Taito")?),
            Hudson => Ok(write!(f, "Hudson")?),
            Banpresto => Ok(write!(f, "Banpresto")?),
            UbiSoft => Ok(write!(f, "Ubi Soft")?),
            Atlus => Ok(write!(f, "Atlus")?),
            Malibu => Ok(write!(f, "Malibu")?),
            Angel => Ok(write!(f, "angel")?),
            BulletProof => Ok(write!(f, "Bullet-Proof")?),
            Irem => Ok(write!(f, "irem")?),
            Absolute => Ok(write!(f, "Absolute")?),
            Acclaim => Ok(write!(f, "Acclaim")?),
            Activision => Ok(write!(f, "Activision")?),
            AmericanSammy => Ok(write!(f, "American sammy")?),
            HiTechEntertainment => Ok(write!(f, "Hi tech entertainment")?),
            Ljn => Ok(write!(f, "LJN")?),
            Matchbox => Ok(write!(f, "Matchbox")?),
            Mattel => Ok(write!(f, "Mattel")?),
            MiltonBradley => Ok(write!(f, "Milton Bradley")?),
            Titus => Ok(write!(f, "Titus")?),
            Virgin => Ok(write!(f, "Virgin")?),
            LucasArts => Ok(write!(f, "LucasArts")?),
            Ocean => Ok(write!(f, "Ocean")?),
            Interplay => Ok(write!(f, "Interplay")?),
            Broderbund => Ok(write!(f, "Broderbund")?),
            Sculptured => Ok(write!(f, "sculptured")?),
            Sci => Ok(write!(f, "sci")?),
            Thq => Ok(write!(f, "THQ")?),
            Accolade => Ok(write!(f, "Accolade")?),
            Misawa => Ok(write!(f, "misawa")?),
            Lozc => Ok(write!(f, "lozc")?),
            TokumaShotenIntermedia => Ok(write!(f, "Tokuma Shoten Intermedia")?),
            TsukudaOriginal => Ok(write!(f, "Tsukuda Original")?),
            Chunsoft => Ok(write!(f, "Chunsoft")?),
            VideoSystem => Ok(write!(f, "Video system")?),
            Varie => Ok(write!(f, "Varie")?),
            YonezawasPal => Ok(write!(f, "Yonezawa/s’pal")?),
            Kaneko => Ok(write!(f, "Kaneko")?),
            PackInSoft => Ok(write!(f, "Pack in soft")?),
            KonamiYuGiOh => Ok(write!(f, "Konami (Yu-Gi-Oh!)")?),
            Unknown => Ok(write!(f, "Unknown")?),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum OldLicenseeCode {
    None = 0x00,
    Nintendo = 0x01,
    Capcom = 0x08,
    HotB = 0x09,
    Jaleco = 0x0A,
    CoconutsJapan = 0x0B,
    EliteSystems = 0x0C,
    ElectronicArts = 0x13,
    Hudsonsoft = 0x18,
    ItcEntertainment = 0x19,
    Yanoman = 0x1A,
    JapanClary = 0x1D,
    VirginInteractive = 0x1F,
    PcmComplete = 0x24,
    SanX = 0x25,
    KotobukiSystems = 0x28,
    Seta = 0x29,
    Infogrames = 0x30,
    // Nintendo = 0x31,
    Bandai = 0x32,
    NewLicenseeCode = 0x33,
    Konami = 0x34,
    HectorSoft = 0x35,
    // Capcom = 0x38,
    Banpresto = 0x39,
    EntertainmentI = 0x3C,
    Gremlin = 0x3E,
    Ubisoft = 0x41,
    Atlus = 0x42,
    Malibu = 0x44,
    Angel = 0x46,
    SpectrumHoloby = 0x47,
    Irem = 0x49,
    // VirginInteractive = 0x4A,
    // Malibu = 0x4D,
    USGold = 0x4F,
    Absolute = 0x50,
    Acclaim = 0x51,
    Activision = 0x52,
    AmericanSammy = 0x53,
    GameTek = 0x54,
    ParkPlace = 0x55,
    Ljn = 0x56,
    Matchbox = 0x57,
    MiltonBradley = 0x59,
    Mindscape = 0x5A,
    Romstar = 0x5B,
    NaxatSoft = 0x5C,
    Tradewest = 0x5D,
    Titus = 0x60,
    // VirginInteractive = 0x61,
    OceanInteractive = 0x67,
    // 	ElectronicArts  0x69,
    // EliteSystems = 0x6E,
    ElectroBrain = 0x6F,
    // Infogrames = 0x70,
    Interplay = 0x71,
    Broderbund = 0x72,
    SculpteredSoft = 0x73,
    TheSalesCurve = 0x75,
    Thq = 0x78,
    Accolade = 0x79,
    TriffixEntertainment = 0x7A,
    Microprose = 0x7C,
    Kemco = 0x7F,
    MisawaEntertainment = 0x80,
    Lozc = 0x83,
    TokumaShotenIntermedia = 0x86,
    BulletProofSoftware = 0x8B,
    VicTokai = 0x8C,
    Ape = 0x8E,
    IMax = 0x8F,
    ChunsoftCo = 0x91,
    VideoSystem = 0x92,
    TsubarayaProductionsCo = 0x93,
    VarieCorporation = 0x95,
    YonezawasPal = 0x96,
    Kaneko = 0x97,
    Arc = 0x99,
    NihonBussan = 0x9A,
    Tecmo = 0x9B,
    Imagineer = 0x9C,
    // Banpresto = 0x9D,
    Nova = 0x9F,
    HoriElectric = 0xA1,
    // Bandai = 0xA2,
    // Konami = 0xA4,
    Kawada = 0xA6,
    Takara = 0xA7,
    TechnosJapan = 0xA9,
    // Broderbund = 0xAA,
    ToeiAnimation = 0xAC,
    Toho = 0xAD,
    Namco = 0xAF,
    // Acclaim = 0xB0,
    AsciiOrNexsoft = 0xB1,
    // Bandai = 0xB2,
    SquareEnix = 0xB4,
    HALLaboratory = 0xB6,
    Snk = 0xB7,
    PonyCanyon = 0xB9,
    CultureBrain = 0xBA,
    Sunsoft = 0xBB,
    SonyImagesoft = 0xBD,
    Sammy = 0xBF,
    Taito = 0xC0,
    // Kemco = 0xC2,
    Squaresoft = 0xC3,
    // TokumaShotenIntermedia = 0xC4,
    DataEast = 0xC5,
    Tonkinhouse = 0xC6,
    Koei = 0xC8,
    Ufl = 0xC9,
    Ultra = 0xCA,
    Vap = 0xCB,
    UseCorporation = 0xCC,
    Meldac = 0xCD,
    PonyCanyonOr = 0xCE,
    // Angel = 0xCF,
    // Taito = 0xD0,
    Sofel = 0xD1,
    Quest = 0xD2,
    SigmaEnterprises = 0xD3,
    AskKodanshaCo = 0xD4,
    // NaxatSoft = 0xD6,
    CopyaSystem = 0xD7,
    // Banpresto = 0xD9,
    Tomy = 0xDA,
    // Ljn = 0xDB,
    Ncs = 0xDD,
    Human = 0xDE,
    Altron = 0xDF,
    // Jaleco = 0xE0,
    TowaChiki = 0xE1,
    Yutaka = 0xE2,
    Varie = 0xE3,
    Epcoh = 0xE5,
    Athena = 0xE7,
    AsmikAceEntertainment = 0xE8,
    Natsume = 0xE9,
    KingRecords = 0xEA,
    // Atlus = 0xEB,
    EpicSonyRecords = 0xEC,
    Igs = 0xEE,
    AWave = 0xF0,
    ExtremeEntertainment = 0xF3,
    // Ljn = 0xFF,
    Unknown,
}

impl From<u8> for OldLicenseeCode {
    fn from(value: u8) -> Self {
        use OldLicenseeCode::*;
        match value {
            0x00 => None,
            0x01 => Nintendo,
            0x08 => Capcom,
            0x09 => HotB,
            0x0A => Jaleco,
            0x0B => CoconutsJapan,
            0x0C => EliteSystems,
            0x13 => ElectronicArts,
            0x18 => Hudsonsoft,
            0x19 => ItcEntertainment,
            0x1A => Yanoman,
            0x1D => JapanClary,
            0x1F => VirginInteractive,
            0x24 => PcmComplete,
            0x25 => SanX,
            0x28 => KotobukiSystems,
            0x29 => Seta,
            0x30 => Infogrames,
            0x31 => Nintendo,
            0x32 => Bandai,
            0x33 => NewLicenseeCode,
            0x34 => Konami,
            0x35 => HectorSoft,
            0x38 => Capcom,
            0x39 => Banpresto,
            0x3C => EntertainmentI,
            0x3E => Gremlin,
            0x41 => Ubisoft,
            0x42 => Atlus,
            0x44 => Malibu,
            0x46 => Angel,
            0x47 => SpectrumHoloby,
            0x49 => Irem,
            0x4A => VirginInteractive,
            0x4D => Malibu,
            0x4F => USGold,
            0x50 => Absolute,
            0x51 => Acclaim,
            0x52 => Activision,
            0x53 => AmericanSammy,
            0x54 => GameTek,
            0x55 => ParkPlace,
            0x56 => Ljn,
            0x57 => Matchbox,
            0x59 => MiltonBradley,
            0x5A => Mindscape,
            0x5B => Romstar,
            0x5C => NaxatSoft,
            0x5D => Tradewest,
            0x60 => Titus,
            0x61 => VirginInteractive,
            0x67 => OceanInteractive,
            0x69 => ElectronicArts,
            0x6E => EliteSystems,
            0x6F => ElectroBrain,
            0x70 => Infogrames,
            0x71 => Interplay,
            0x72 => Broderbund,
            0x73 => SculpteredSoft,
            0x75 => TheSalesCurve,
            0x78 => Thq,
            0x79 => Accolade,
            0x7A => TriffixEntertainment,
            0x7C => Microprose,
            0x7F => Kemco,
            0x80 => MisawaEntertainment,
            0x83 => Lozc,
            0x86 => TokumaShotenIntermedia,
            0x8B => BulletProofSoftware,
            0x8C => VicTokai,
            0x8E => Ape,
            0x8F => IMax,
            0x91 => ChunsoftCo,
            0x92 => VideoSystem,
            0x93 => TsubarayaProductionsCo,
            0x95 => VarieCorporation,
            0x96 => YonezawasPal,
            0x97 => Kaneko,
            0x99 => Arc,
            0x9A => NihonBussan,
            0x9B => Tecmo,
            0x9C => Imagineer,
            0x9D => Banpresto,
            0x9F => Nova,
            0xA1 => HoriElectric,
            0xA2 => Bandai,
            0xA4 => Konami,
            0xA6 => Kawada,
            0xA7 => Takara,
            0xA9 => TechnosJapan,
            0xAA => Broderbund,
            0xAC => ToeiAnimation,
            0xAD => Toho,
            0xAF => Namco,
            0xB0 => Acclaim,
            0xB1 => AsciiOrNexsoft,
            0xB2 => Bandai,
            0xB4 => SquareEnix,
            0xB6 => HALLaboratory,
            0xB7 => Snk,
            0xB9 => PonyCanyon,
            0xBA => CultureBrain,
            0xBB => Sunsoft,
            0xBD => SonyImagesoft,
            0xBF => Sammy,
            0xC0 => Taito,
            0xC2 => Kemco,
            0xC3 => Squaresoft,
            0xC4 => TokumaShotenIntermedia,
            0xC5 => DataEast,
            0xC6 => Tonkinhouse,
            0xC8 => Koei,
            0xC9 => Ufl,
            0xCA => Ultra,
            0xCB => Vap,
            0xCC => UseCorporation,
            0xCD => Meldac,
            0xCE => PonyCanyonOr,
            0xCF => Angel,
            0xD0 => Taito,
            0xD1 => Sofel,
            0xD2 => Quest,
            0xD3 => SigmaEnterprises,
            0xD4 => AskKodanshaCo,
            0xD6 => NaxatSoft,
            0xD7 => CopyaSystem,
            0xD9 => Banpresto,
            0xDA => Tomy,
            0xDB => Ljn,
            0xDD => Ncs,
            0xDE => Human,
            0xDF => Altron,
            0xE0 => Jaleco,
            0xE1 => TowaChiki,
            0xE2 => Yutaka,
            0xE3 => Varie,
            0xE5 => Epcoh,
            0xE7 => Athena,
            0xE8 => AsmikAceEntertainment,
            0xE9 => Natsume,
            0xEA => KingRecords,
            0xEB => Atlus,
            0xEC => EpicSonyRecords,
            0xEE => Igs,
            0xF0 => AWave,
            0xF3 => ExtremeEntertainment,
            0xFF => Ljn,
            _ => Unknown,
        }
    }
}

impl Display for OldLicenseeCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use OldLicenseeCode::*;
        match self {
            None => Ok(write!(f, "None")?),
            Nintendo => Ok(write!(f, "Nintendo")?),
            Capcom => Ok(write!(f, "Capcom")?),
            HotB => Ok(write!(f, "Hot-B")?),
            Jaleco => Ok(write!(f, "Jaleco")?),
            CoconutsJapan => Ok(write!(f, "Coconuts Japan")?),
            EliteSystems => Ok(write!(f, "Elite Systems")?),
            ElectronicArts => Ok(write!(f, "Electronic Arts")?),
            Hudsonsoft => Ok(write!(f, "Hudsonsoft")?),
            ItcEntertainment => Ok(write!(f, "ITC Entertainment")?),
            Yanoman => Ok(write!(f, "Yanoman")?),
            JapanClary => Ok(write!(f, "Japan Clary")?),
            VirginInteractive => Ok(write!(f, "Virgin Interactive")?),
            PcmComplete => Ok(write!(f, "PCM Complete")?),
            SanX => Ok(write!(f, "San-X")?),
            KotobukiSystems => Ok(write!(f, "Kotobuki Systems")?),
            Seta => Ok(write!(f, "Seta")?),
            Infogrames => Ok(write!(f, "Infogrames")?),
            Bandai => Ok(write!(f, "Bandai")?),
            NewLicenseeCode => Ok(write!(f, "New Licensee Code")?),
            Konami => Ok(write!(f, "Konami")?),
            HectorSoft => Ok(write!(f, "HectorSoft")?),
            Banpresto => Ok(write!(f, "Banpresto")?),
            EntertainmentI => Ok(write!(f, ".Entertainment i")?),
            Gremlin => Ok(write!(f, "Gremlin")?),
            Ubisoft => Ok(write!(f, "Ubisoft")?),
            Atlus => Ok(write!(f, "Atlus")?),
            Malibu => Ok(write!(f, "Malibu")?),
            Angel => Ok(write!(f, "Angel")?),
            SpectrumHoloby => Ok(write!(f, "Spectrum Holoby")?),
            Irem => Ok(write!(f, "Irem")?),
            USGold => Ok(write!(f, "U.S. Gold")?),
            Absolute => Ok(write!(f, "Absolute")?),
            Acclaim => Ok(write!(f, "Acclaim")?),
            Activision => Ok(write!(f, "Activision")?),
            AmericanSammy => Ok(write!(f, "American Sammy")?),
            GameTek => Ok(write!(f, "GameTek")?),
            ParkPlace => Ok(write!(f, "Park Place")?),
            Ljn => Ok(write!(f, "LJN")?),
            Matchbox => Ok(write!(f, "Matchbox")?),
            MiltonBradley => Ok(write!(f, "Milton Bradley")?),
            Mindscape => Ok(write!(f, "Mindscape")?),
            Romstar => Ok(write!(f, "Romstar")?),
            NaxatSoft => Ok(write!(f, "Naxat Soft")?),
            Tradewest => Ok(write!(f, "Tradewest")?),
            Titus => Ok(write!(f, "Titus")?),
            OceanInteractive => Ok(write!(f, "Ocean Interactive")?),
            ElectroBrain => Ok(write!(f, "Electro Brain")?),
            Interplay => Ok(write!(f, "Interplay")?),
            Broderbund => Ok(write!(f, "Broderbund")?),
            SculpteredSoft => Ok(write!(f, "Sculptered Soft")?),
            TheSalesCurve => Ok(write!(f, "The Sales Curve")?),
            Thq => Ok(write!(f, "THQ")?),
            Accolade => Ok(write!(f, "Accolade")?),
            TriffixEntertainment => Ok(write!(f, "Triffix Entertainment")?),
            Microprose => Ok(write!(f, "Microprose")?),
            Kemco => Ok(write!(f, "Kemco")?),
            MisawaEntertainment => Ok(write!(f, "Misawa Entertainment")?),
            Lozc => Ok(write!(f, "Lozc")?),
            TokumaShotenIntermedia => Ok(write!(f, "Tokuma Shoten Intermedia")?),
            BulletProofSoftware => Ok(write!(f, "Bullet-Proof Software")?),
            VicTokai => Ok(write!(f, "Vic Tokai")?),
            Ape => Ok(write!(f, "Ape")?),
            IMax => Ok(write!(f, "I'max")?),
            ChunsoftCo => Ok(write!(f, "Chunsoft Co.")?),
            VideoSystem => Ok(write!(f, "Video System")?),
            TsubarayaProductionsCo => Ok(write!(f, "Tsubaraya Productions Co.")?),
            VarieCorporation => Ok(write!(f, "Varie Corporation")?),
            YonezawasPal => Ok(write!(f, "Yonezawa/S'Pal")?),
            Kaneko => Ok(write!(f, "Kaneko")?),
            Arc => Ok(write!(f, "Arc")?),
            NihonBussan => Ok(write!(f, "Nihon Bussan")?),
            Tecmo => Ok(write!(f, "Tecmo")?),
            Imagineer => Ok(write!(f, "Imagineer")?),
            Nova => Ok(write!(f, "Nova")?),
            HoriElectric => Ok(write!(f, "Hori Electric")?),
            Kawada => Ok(write!(f, "Kawada")?),
            Takara => Ok(write!(f, "Takara")?),
            TechnosJapan => Ok(write!(f, "Technos Japan")?),
            ToeiAnimation => Ok(write!(f, "Toei Animation")?),
            Toho => Ok(write!(f, "Toho")?),
            Namco => Ok(write!(f, "Namco")?),
            AsciiOrNexsoft => Ok(write!(f, "ASCII or Nexsoft")?),
            SquareEnix => Ok(write!(f, "Square Enix")?),
            HALLaboratory => Ok(write!(f, "HAL Laboratory")?),
            Snk => Ok(write!(f, "SNK")?),
            PonyCanyon => Ok(write!(f, "Pony Canyon")?),
            CultureBrain => Ok(write!(f, "Culture Brain")?),
            Sunsoft => Ok(write!(f, "Sunsoft")?),
            SonyImagesoft => Ok(write!(f, "Sony Imgaesoft")?),
            Sammy => Ok(write!(f, "Sammy")?),
            Taito => Ok(write!(f, "Taito")?),
            Squaresoft => Ok(write!(f, "Squaresoft")?),
            DataEast => Ok(write!(f, "Data East")?),
            Tonkinhouse => Ok(write!(f, "Tonkinhouse")?),
            Koei => Ok(write!(f, "Koei")?),
            Ufl => Ok(write!(f, "Ufl")?),
            Ultra => Ok(write!(f, "Ultra")?),
            Vap => Ok(write!(f, "Vap")?),
            UseCorporation => Ok(write!(f, "Use Corporation")?),
            Meldac => Ok(write!(f, "Meldac")?),
            PonyCanyonOr => Ok(write!(f, ".Pony Canyon Or")?),
            Sofel => Ok(write!(f, "Sofel")?),
            Quest => Ok(write!(f, "Quest")?),
            SigmaEnterprises => Ok(write!(f, "Sigma Enterprises")?),
            AskKodanshaCo => Ok(write!(f, "ASK Kodansha Co.")?),
            CopyaSystem => Ok(write!(f, "Copya System")?),
            Tomy => Ok(write!(f, "Tomy")?),
            Ncs => Ok(write!(f, "NCS")?),
            Human => Ok(write!(f, "Human")?),
            Altron => Ok(write!(f, "Altron")?),
            TowaChiki => Ok(write!(f, "Towa Chiki")?),
            Yutaka => Ok(write!(f, "Yutaka")?),
            Varie => Ok(write!(f, "Varie")?),
            Epcoh => Ok(write!(f, "Epcoh")?),
            Athena => Ok(write!(f, "Athena")?),
            AsmikAceEntertainment => Ok(write!(f, "Asmik ACE Entertainment")?),
            Natsume => Ok(write!(f, "Natsume")?),
            KingRecords => Ok(write!(f, "King Records")?),
            EpicSonyRecords => Ok(write!(f, "Epic/Sony Records")?),
            Igs => Ok(write!(f, "Igs")?),
            AWave => Ok(write!(f, "A Wave")?),
            ExtremeEntertainment => Ok(write!(f, "Extreme Entertainement")?),
            Unknown => Ok(write!(f, "Unknown")?),
        }
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
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
    use crate::{types::cartrige::{get_rom_size, ColorMode, Destination}, Device};

    #[test]
    fn test_rom_size() {
        assert_eq!(get_rom_size(0x00), 0x8000);
        assert_eq!(get_rom_size(0x01), 0x10000);
        assert_eq!(get_rom_size(0x05), 0x100000);
        assert_eq!(get_rom_size(0x54), 0x180000);
    }

    #[test]
    fn read_header() {
        let mut d = Device::new();
        d.load_cartrige("../roms/smbd.gbc").unwrap();
        let c = d.get_cartridge_header().unwrap();
        assert_eq!(c.cgb_flag, ColorMode::ColorOnly);
        assert_eq!(c.destination, Destination::NotJapan);    
        assert_eq!(c.licensee_code, 0x31);
    }
}
