use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug, Clone)]
pub struct INesHeader {
    name: [u8; 4],
    pub prg_rom_size: u8,
    pub chr_rom_size: u8,
    pub mapper_1: u8,
    pub mapper_2: u8,
    pub prg_ram_size: u8,
    pub tv_system_1: u8,
    pub tv_system_2: u8,
    _unused: [u8; 5],
}

impl std::fmt::Display for INesHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mapper = (self.mapper_2 & 0xF0) | (self.mapper_1 >> 4);
        write!(f, "Name: {:?}\nPRG ROM Size: {:?}\nCHR ROM Size: {:?}\nMapper: {:?}\n", self.name, self.prg_rom_size, self.chr_rom_size, mapper)
    }
}

#[derive(Debug, Clone)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    FourScreen,
}

impl std::fmt::Display for Mirroring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mirroring::Horizontal => write!(f, "Horizontal"),
            Mirroring::Vertical => write!(f, "Vertical"),
            Mirroring::FourScreen => write!(f, "FourScreen"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cartridge {
    pub header: INesHeader,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub chr_is_ram: bool,
    pub mirror: Mirroring,
    pub mapper: u8,
}

impl Cartridge {
    pub fn new(filename: &str) -> Result<Cartridge, String> {
        let mut file = File::open(filename).unwrap();
        let mut header_buffer: Vec<u8> = vec![0; 16];

        file.read_exact(&mut header_buffer).unwrap();

        let header = INesHeader {
            name: [header_buffer[0], header_buffer[1], header_buffer[2], header_buffer[3]],
            prg_rom_size: header_buffer[4],
            chr_rom_size: header_buffer[5],
            mapper_1: header_buffer[6],
            mapper_2: header_buffer[7],
            prg_ram_size: header_buffer[8],
            tv_system_1: header_buffer[9],
            tv_system_2: header_buffer[10],
            _unused: [header_buffer[11], header_buffer[12], header_buffer[13], header_buffer[14], header_buffer[15]],
        };

        if header.name != [0x4E, 0x45, 0x53, 0x1A] {
            return Err("File is not in iNES file format".to_string());
        }

        // Skip the trainer data if header.mapper_1 is 0x04
        if header.mapper_1 & 0x04 == 0x04 {
            file.seek(SeekFrom::Current(512)).unwrap();
        }

        let mapper = (header.mapper_2 & 0xF0) | (header.mapper_1 >> 4);

        let prg_bank_size = 16384 * header.prg_rom_size as usize;
        let mut prg_rom = vec![0; prg_bank_size];
        file.read_exact(&mut prg_rom).unwrap();

        let chr_is_ram = header.chr_rom_size == 0;
        let chr_bank_size = if chr_is_ram {
            8192
        } else {
            8192 * header.chr_rom_size as usize
        };
        let mut chr_rom = vec![0; chr_bank_size];
        if !chr_is_ram {
            file.read_exact(&mut chr_rom).unwrap();
        }

        // Mirroing
        let four_screen = header.mapper_1 & 0x08 == 0x08;
        let vertical = header.mapper_1 & 0x01 == 0x01;

        let mirror = match (four_screen, vertical) {
            (true, _) => Mirroring::FourScreen,
            (false, true) => Mirroring::Vertical,
            (false, false) => Mirroring::Horizontal,
        };

        Ok(Cartridge {
            header,
            prg_rom,
            chr_rom,
            chr_is_ram,
            mirror,
            mapper,
        })
    }
}
