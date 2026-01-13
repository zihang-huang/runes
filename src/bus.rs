use crate::cartridge::Cartridge;
use crate::ppu::PPU;


// Memory addresses
const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

pub struct Bus {
    pub cpu_vram: [u8; 2048],
    pub cartridge: Cartridge,
    pub ppu: PPU,
    controller: [u8; 2],
    controller_state: [u8; 2],
    controller_strobe: bool,
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Bus {
        Bus {
            cpu_vram: [0; 2048],
            ppu: PPU::new(
                cartridge.chr_rom.clone(),
                cartridge.mirror.clone(),
                cartridge.chr_is_ram,
            ),
            cartridge,
            controller: [0; 2],
            controller_state: [0; 2],
            controller_strobe: false,
        }
    }
}

impl Bus {
    pub fn set_controller_state(&mut self, index: usize, state: u8) {
        if let Some(slot) = self.controller.get_mut(index) {
            *slot = state;
            if self.controller_strobe {
                self.controller_state[index] = state;
            }
        }
    }

    pub fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0x07FF;                
                self.cpu_vram[mirror_down_addr as usize]
            },

            // PPU
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 => 0,
            0x2002 => self.ppu.read_status_register(),

            0x2004 => self.ppu.read_oam_data(),

            0x2007 => self.ppu.read_data(),

            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = 0x2000 + (addr & 0x0007);
                self.mem_read(mirror_down_addr)
            },

            0x4014 => 0,

            0x4016 | 0x4017 => {
                let index = (addr & 0x0001) as usize;
                let value = self.controller_state[index] & 0x01;
                if !self.controller_strobe {
                    self.controller_state[index] >>= 1;
                }
                value
            },

            // ROM(Cartridge)
            0x8000..=0xFFFF => self.read_prg_rom(addr),
            

            _ => {
                println!("Unmapped memory address: {:#X}", addr);
                0
            }

        }
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0x07FF;
                self.cpu_vram[mirror_down_addr as usize] = data;
            },

            0x2000 => self.ppu.write_to_control_register(data),

            0x2001 => self.ppu.write_to_mask_register(data),

            0x2002 => {},

            0x2003 => self.ppu.write_to_oam_address(data),

            0x2004 => self.ppu.write_to_oam_data(data),

            0x2005 => self.ppu.write_to_scroll_register(data),

            0x2006 => self.ppu.write_to_address_register(data),

            0x2007 => self.ppu.write_data(data),

            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = 0x2000 + (addr & 0x0007);
                self.mem_write(mirror_down_addr, data);
            },

            0x4014 => {
                let base = (data as u16) << 8;
                for i in 0..=255u16 {
                    let value = self.mem_read(base + i);
                    self.ppu.write_to_oam_data(value);
                }
            },

            0x4016 => {
                self.controller_strobe = data & 0x01 == 0x01;
                if self.controller_strobe {
                    self.controller_state = self.controller;
                }
            },

            0x4017 => {},

            0x8000..=0xFFFF => {
                // Mapper not implemented yet.
            },

            _ => {
                println!("Unmapped memory address: {:#X}", addr);
            }

        }
    }

    pub fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr -= 0x8000;
        if self.cartridge.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            // Mirror
            addr -= 0x4000;
        }

        self.cartridge.prg_rom[addr as usize]
    }
    
}
