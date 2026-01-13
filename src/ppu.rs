use crate::cartridge::Mirroring;

pub enum PPUStatusFlags {
    SpriteOverflow = (1 << 5),
    SpriteZeroHit = (1 << 6),
    VerticalBlank = (1 << 7),
}

pub enum PPUControlFlags {
    NametableX = (1 << 0),
    NametableY = (1 << 1),
    IncrementMode = (1 << 2),
    PatternSprite = (1 << 3),
    PatternBackground = (1 << 4),
    SpriteSize = (1 << 5),
    SlaveMode = (1 << 6),
    EnableNMI = (1 << 7),
}

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 240;

// In the format of (R,G,B)
pub static SYSTEM_PALLETE: [(u8, u8, u8); 64] = [
    (0x80, 0x80, 0x80),
    (0x00, 0x3D, 0xA6),
    (0x00, 0x12, 0xB0),
    (0x44, 0x00, 0x96),
    (0xA1, 0x00, 0x5E),
    (0xC7, 0x00, 0x28),
    (0xBA, 0x06, 0x00),
    (0x8C, 0x17, 0x00),
    (0x5C, 0x2F, 0x00),
    (0x10, 0x45, 0x00),
    (0x05, 0x4A, 0x00),
    (0x00, 0x47, 0x2E),
    (0x00, 0x41, 0x66),
    (0x00, 0x00, 0x00),
    (0x05, 0x05, 0x05),
    (0x05, 0x05, 0x05),
    (0xC7, 0xC7, 0xC7),
    (0x00, 0x77, 0xFF),
    (0x21, 0x55, 0xFF),
    (0x82, 0x37, 0xFA),
    (0xEB, 0x2F, 0xB5),
    (0xFF, 0x29, 0x50),
    (0xFF, 0x22, 0x00),
    (0xD6, 0x32, 0x00),
    (0xC4, 0x62, 0x00),
    (0x35, 0x80, 0x00),
    (0x05, 0x8F, 0x00),
    (0x00, 0x8A, 0x55),
    (0x00, 0x99, 0xCC),
    (0x21, 0x21, 0x21),
    (0x09, 0x09, 0x09),
    (0x09, 0x09, 0x09),
    (0xFF, 0xFF, 0xFF),
    (0x0F, 0xD7, 0xFF),
    (0x69, 0xA2, 0xFF),
    (0xD4, 0x80, 0xFF),
    (0xFF, 0x45, 0xF3),
    (0xFF, 0x61, 0x8B),
    (0xFF, 0x88, 0x33),
    (0xFF, 0x9C, 0x12),
    (0xFA, 0xBC, 0x20),
    (0x9F, 0xE3, 0x0E),
    (0x2B, 0xF0, 0x35),
    (0x0C, 0xF0, 0xA4),
    (0x05, 0xFB, 0xFF),
    (0x5E, 0x5E, 0x5E),
    (0x0D, 0x0D, 0x0D),
    (0x0D, 0x0D, 0x0D),
    (0xFF, 0xFF, 0xFF),
    (0xA6, 0xFC, 0xFF),
    (0xB3, 0xEC, 0xFF),
    (0xDA, 0xAB, 0xEB),
    (0xFF, 0xA8, 0xF9),
    (0xFF, 0xAB, 0xB3),
    (0xFF, 0xD2, 0xB0),
    (0xFF, 0xEF, 0xA6),
    (0xFF, 0xF7, 0x9C),
    (0xD7, 0xE8, 0x95),
    (0xA6, 0xED, 0xAF),
    (0xA2, 0xF2, 0xDA),
    (0x99, 0xFF, 0xFC),
    (0xDD, 0xDD, 0xDD),
    (0x11, 0x11, 0x11),
    (0x11, 0x11, 0x11),
];

pub struct PPU {
    pub chr_rom: Vec<u8>,
    pub vram: Vec<u8>,
    pub oam: [u8; 256],
    pub palette: [u8; 32],

    // PPU Registers
    pub address_register: u16,
    address_latch: bool,

    pub control_register: u8,
    pub nmi: bool,

    pub mask_register: u8,

    pub status_register: u8,

    // Data Buffer
    pub data_buffer: u8,

    pub mirroring: Mirroring,

    // Miscs
    pub scanline: u16,
    pub cycle: u16,

    pub frame_complete: bool,
    pub frame_buffer: Vec<u8>,
    background_index_buffer: Vec<u8>,

    chr_is_ram: bool,
    oam_addr: u8,
    scroll_x: u8,
    scroll_y: u8,
}

impl PPU {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring, chr_is_ram: bool) -> PPU {
        let vram_size = match mirroring {
            Mirroring::FourScreen => 0x1000,
            _ => 0x0800,
        };

        PPU {
            chr_rom,
            vram: vec![0; vram_size],
            oam: [0xFF; 256],
            palette: [0; 32],
            address_register: 0,
            address_latch: true,

            control_register: 0,
            nmi: false,

            mask_register: 0,

            status_register: 0,

            data_buffer: 0,

            mirroring,

            scanline: 0,
            cycle: 0,

            frame_complete: false,
            frame_buffer: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT * 3],
            background_index_buffer: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT],

            chr_is_ram,
            oam_addr: 0,
            scroll_x: 0,
            scroll_y: 0,
        }
    }

    pub fn reset(&mut self) {
        self.address_register = 0;
        self.address_latch = true;
        self.control_register = 0;
        self.mask_register = 0;
        self.status_register = 0;
        self.data_buffer = 0;
        self.oam.fill(0xFF);
        self.scanline = 0;
        self.cycle = 0;
        self.nmi = false;
        self.frame_complete = false;
        self.oam_addr = 0;
        self.scroll_x = 0;
        self.scroll_y = 0;
        self.frame_buffer.fill(0);
        self.background_index_buffer.fill(0);
    }

    // Mirroring
    pub fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0x2FFF;
        let vram_index = mirrored_vram - 0x2000;
        let name_table = vram_index / 0x0400;

        match self.mirroring {
            Mirroring::Vertical => match name_table {
                2 | 3 => vram_index - 0x0800,
                _ => vram_index,
            },
            Mirroring::Horizontal => match name_table {
                1 | 2 => vram_index - 0x0400,
                3 => vram_index - 0x0800,
                _ => vram_index,
            },
            Mirroring::FourScreen => vram_index,
        }
    }

    // Address Register
    pub fn write_to_address_register(&mut self, data: u8) {
        if self.address_latch {
            self.address_register = (self.address_register & 0x00FF) | ((data as u16) << 8);
        } else {
            self.address_register = (self.address_register & 0xFF00) | (data as u16);
        }

        self.address_register &= 0x3FFF;
        self.address_latch = !self.address_latch;
    }

    pub fn increment_address_register(&mut self, increment: u8) {
        self.address_register = self.address_register.wrapping_add(increment as u16);
        self.address_register &= 0x3FFF;
    }

    pub fn reset_address_latch(&mut self) {
        self.address_latch = true;
    }

    // Control Register
    pub fn write_to_control_register(&mut self, data: u8) {
        self.control_register = data;
    }

    pub fn get_control_flag(&self, flag: PPUControlFlags) -> bool {
        self.control_register & (flag as u8) != 0
    }

    pub fn increment_vram_addr(&mut self) {
        let increment: u8 = if self.control_register & PPUControlFlags::IncrementMode as u8 == 0 {
            1
        } else {
            32
        };

        self.increment_address_register(increment);
    }

    // Status register
    pub fn read_status_register(&mut self) -> u8 {
        let status = (self.status_register & 0xE0) | (self.data_buffer & 0x1F); // Noise
        self.set_status_flag(PPUStatusFlags::VerticalBlank, false);
        self.address_latch = true;
        status
    }

    pub fn set_status_flag(&mut self, flag: PPUStatusFlags, value: bool) {
        if value {
            self.status_register |= flag as u8;
        } else {
            self.status_register &= !(flag as u8);
        }
    }

    // Mask Register
    pub fn write_to_mask_register(&mut self, data: u8) {
        self.mask_register = data;
    }

    pub fn write_to_scroll_register(&mut self, data: u8) {
        if self.address_latch {
            self.scroll_x = data;
        } else {
            self.scroll_y = data;
        }
        self.address_latch = !self.address_latch;
    }

    pub fn write_to_oam_address(&mut self, data: u8) {
        self.oam_addr = data;
    }

    pub fn write_to_oam_data(&mut self, data: u8) {
        self.oam[self.oam_addr as usize] = data;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    pub fn read_oam_data(&self) -> u8 {
        self.oam[self.oam_addr as usize]
    }

    // PPU Read & Write
    pub fn read_data(&mut self) -> u8 {
        let addr = self.address_register;
        let data = match addr {
            0x3F00..=0x3FFF => {
                let value = self.ppu_read(addr);
                self.data_buffer = self.ppu_read(addr - 0x1000);
                value
            }
            _ => {
                let value = self.data_buffer;
                self.data_buffer = self.ppu_read(addr);
                value
            }
        };

        self.increment_vram_addr();
        data
    }

    pub fn write_data(&mut self, data: u8) {
        let addr = self.address_register;
        self.ppu_write(addr, data);
        self.increment_vram_addr();
    }

    fn ppu_read(&self, addr: u16) -> u8 {
        let addr = addr & 0x3FFF;
        match addr {
            0x0000..=0x1FFF => self.chr_rom[addr as usize],
            0x2000..=0x2FFF => {
                let index = self.mirror_vram_addr(addr) as usize;
                self.vram[index]
            }
            0x3000..=0x3EFF => self.ppu_read(addr - 0x1000),
            0x3F00..=0x3FFF => {
                let mut palette_index = (addr - 0x3F00) % 32;
                palette_index = match palette_index {
                    0x10 => 0x00,
                    0x14 => 0x04,
                    0x18 => 0x08,
                    0x1C => 0x0C,
                    _ => palette_index,
                };
                self.palette[palette_index as usize] & 0x3F
            }
            _ => 0,
        }
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {
        let addr = addr & 0x3FFF;
        match addr {
            0x0000..=0x1FFF => {
                if self.chr_is_ram {
                    self.chr_rom[addr as usize] = data;
                }
            }
            0x2000..=0x2FFF => {
                let index = self.mirror_vram_addr(addr) as usize;
                self.vram[index] = data;
            }
            0x3000..=0x3EFF => self.ppu_write(addr - 0x1000, data),
            0x3F00..=0x3FFF => {
                let mut palette_index = (addr - 0x3F00) % 32;
                palette_index = match palette_index {
                    0x10 => 0x00,
                    0x14 => 0x04,
                    0x18 => 0x08,
                    0x1C => 0x0C,
                    _ => palette_index,
                };
                self.palette[palette_index as usize] = data & 0x3F;
            }
            _ => {}
        }
    }

    fn set_frame_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) {
        if x >= SCREEN_WIDTH || y >= SCREEN_HEIGHT {
            return;
        }

        let index = (y * SCREEN_WIDTH + x) * 3;
        self.frame_buffer[index] = rgb.0;
        self.frame_buffer[index + 1] = rgb.1;
        self.frame_buffer[index + 2] = rgb.2;
    }

    fn render_sprites(&mut self) {
        if self.mask_register & 0x10 == 0 {
            return;
        }

        let show_leftmost_sprites = self.mask_register & 0x04 != 0;
        let show_background = self.mask_register & 0x08 != 0;

        let sprite_height = if self.get_control_flag(PPUControlFlags::SpriteSize) {
            16
        } else {
            8
        };

        for sprite_index in 0..64 {
            let base = sprite_index * 4;
            let y = self.oam[base] as i16 + 1;
            let tile_index = self.oam[base + 1];
            let attributes = self.oam[base + 2];
            let x = self.oam[base + 3] as i16;

            let flip_h = attributes & 0x40 != 0;
            let flip_v = attributes & 0x80 != 0;
            let behind_background = attributes & 0x20 != 0;
            let palette_index = attributes & 0x03;

            for row in 0..sprite_height {
                let row_index = if flip_v {
                    sprite_height - 1 - row
                } else {
                    row
                };

                let (pattern_table, tile) = if sprite_height == 16 {
                    let table = if tile_index & 0x01 == 0 { 0x0000 } else { 0x1000 };
                    let mut tile = tile_index & 0xFE;
                    let mut row_in_tile = row_index;
                    if row_in_tile >= 8 {
                        tile = tile.wrapping_add(1);
                        row_in_tile -= 8;
                    }
                    (table, (tile, row_in_tile))
                } else {
                    let table = if self.get_control_flag(PPUControlFlags::PatternSprite) {
                        0x1000
                    } else {
                        0x0000
                    };
                    (table, (tile_index, row_index))
                };

                let tile_addr = pattern_table + (tile.0 as u16) * 16 + tile.1 as u16;
                let plane_low = self.ppu_read(tile_addr);
                let plane_high = self.ppu_read(tile_addr + 8);

                for col in 0..8 {
                    let bit = if flip_h { col } else { 7 - col };
                    let color_low = (plane_low >> bit) & 0x01;
                    let color_high = (plane_high >> bit) & 0x01;
                    let color = (color_high << 1) | color_low;
                    if color == 0 {
                        continue;
                    }

                    let palette_addr = 0x3F10 + (palette_index as u16) * 4 + color as u16;
                    let palette_value = self.ppu_read(palette_addr) & 0x3F;
                    let rgb = SYSTEM_PALLETE[palette_value as usize];

                    let pixel_x = x + col as i16;
                    let pixel_y = y + row as i16;
                    if pixel_x < 0 || pixel_y < 0 {
                        continue;
                    }

                    let pixel_x = pixel_x as usize;
                    let pixel_y = pixel_y as usize;
                    if pixel_x >= SCREEN_WIDTH || pixel_y >= SCREEN_HEIGHT {
                        continue;
                    }

                    if pixel_x < 8 && !show_leftmost_sprites {
                        continue;
                    }

                    if behind_background && show_background {
                        let bg_color =
                            self.background_index_buffer[pixel_y * SCREEN_WIDTH + pixel_x];
                        if bg_color != 0 {
                            continue;
                        }
                    }

                    self.set_frame_pixel(pixel_x, pixel_y, rgb);
                }
            }
        }
    }

    fn background_pixel_info(&self, x: u16, y: u16) -> ((u8, u8, u8), u8) {
        let base_nametable = self.control_register & 0x03;
        let base_x = if base_nametable & 0x01 != 0 { 256 } else { 0 };
        let base_y = if base_nametable & 0x02 != 0 { 240 } else { 0 };

        let world_x = x.wrapping_add(self.scroll_x as u16 + base_x);
        let world_y = y.wrapping_add(self.scroll_y as u16 + base_y);

        let nametable_x = (world_x / 256) % 2;
        let nametable_y = (world_y / 240) % 2;
        let nametable_base = 0x2000 + (nametable_y * 2 + nametable_x) * 0x0400;

        let tile_x = (world_x % 256) / 8;
        let tile_y = (world_y % 240) / 8;
        let tile_index = self.ppu_read(nametable_base + tile_y * 32 + tile_x);

        let attribute_addr = nametable_base + 0x03C0 + (tile_y / 4) * 8 + (tile_x / 4);
        let attribute = self.ppu_read(attribute_addr);
        let quadrant_x = (tile_x % 4) / 2;
        let quadrant_y = (tile_y % 4) / 2;
        let quadrant = quadrant_y * 2 + quadrant_x;

        let palette_select = match quadrant {
            0 => attribute & 0x03,
            1 => (attribute >> 2) & 0x03,
            2 => (attribute >> 4) & 0x03,
            _ => (attribute >> 6) & 0x03,
        };

        let pattern_table_base = if self.get_control_flag(PPUControlFlags::PatternBackground) {
            0x1000
        } else {
            0x0000
        };

        let fine_y = world_y % 8;
        let fine_x = world_x % 8;
        let tile_addr = pattern_table_base + (tile_index as u16) * 16 + fine_y;

        let plane_low = self.ppu_read(tile_addr);
        let plane_high = self.ppu_read(tile_addr + 8);
        let bit = 7 - fine_x;

        let color_low = (plane_low >> bit) & 0x01;
        let color_high = (plane_high >> bit) & 0x01;
        let color = (color_high << 1) | color_low;

        let palette_addr = if color == 0 {
            0x3F00
        } else {
            0x3F00 + ((palette_select << 2) | color) as u16
        };

        let palette_value = self.ppu_read(palette_addr) & 0x3F;
        (SYSTEM_PALLETE[palette_value as usize], color)
    }

    fn background_pixel(&self, x: u16, y: u16) -> (u8, u8, u8) {
        self.background_pixel_info(x, y).0
    }

    pub fn clock(&mut self) {
        if self.scanline < 240 {
            if (1..=256).contains(&self.cycle) {
                let x = (self.cycle - 1) as usize;
                let y = self.scanline as usize;
                let show_background = self.mask_register & 0x08 != 0;
                let show_leftmost_background = self.mask_register & 0x02 != 0;
                let (rgb, bg_color) = if show_background {
                    if x < 8 && !show_leftmost_background {
                        let palette_value = self.ppu_read(0x3F00) & 0x3F;
                        (SYSTEM_PALLETE[palette_value as usize], 0)
                    } else {
                        self.background_pixel_info(x as u16, y as u16)
                    }
                } else {
                    let palette_value = self.ppu_read(0x3F00) & 0x3F;
                    (SYSTEM_PALLETE[palette_value as usize], 0)
                };
                self.background_index_buffer[y * SCREEN_WIDTH + x] = bg_color;
                self.set_frame_pixel(x, y, rgb);
            }
        }

        if self.scanline == 241 && self.cycle == 1 {
            self.set_status_flag(PPUStatusFlags::VerticalBlank, true);

            if self.get_control_flag(PPUControlFlags::EnableNMI) {
                self.nmi = true;
            }
        }

        if self.scanline == 261 && self.cycle == 1 {
            self.set_status_flag(PPUStatusFlags::VerticalBlank, false);
            self.set_status_flag(PPUStatusFlags::SpriteZeroHit, false);
            self.set_status_flag(PPUStatusFlags::SpriteOverflow, false);
        }

        self.cycle += 1;

        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;

            if self.scanline > 261 {
                self.scanline = 0;
                self.render_sprites();
                self.frame_complete = true;
            }
        }
    }
}
