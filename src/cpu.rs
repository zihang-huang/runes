use crate::opcodes::{references, Opcode};
use crate::bus::Bus;
use crate::cartridge::Cartridge;

enum StatusFlag {
    C = (1 << 0), // Carry Bit
    Z = (1 << 1), // Zero
    I = (1 << 2), // Disable Interrupts
    D = (1 << 3), // Decimal Mode (unused in this implementation)
    B = (1 << 4), // Break
    U = (1 << 5), // Unused
    V = (1 << 6), // Overflow
    N = (1 << 7), // Negative
}

#[derive(PartialEq, Debug)]
pub enum AddressingMode {
    IMP, // Implied
    IMM, // Immediate
    ZP0, // Zero Page
    ZPX, // Zero Page with X Offset
    ZPY, // Zero Page with Y Offset
    REL, // Relative
    ABS, // Absolute
    ABX, // Absolute with X Offset
    ABY, // Absolute with Y Offset
    IND, // Indirect
    IZX, // Indirect with X Offset
    IZY, // Indirect with Y Offset
}

pub struct CPU {
    pub accumulator: u8, // Accumulator Register
    pub x_register: u8, // X Register
    pub y_register: u8, // Y Register
    pub stack_pointer: u8, // Stack Pointer (points to location on bus)
    pub program_counter: u16, // Program Counter

    pub status: u8,

    fetched: u8, // Represents the working input value to the ALU

    pub addr_abs: u16, // All used memory addresses end up in here
    pub addr_rel: u16, // Represents absolute address following a branch
    pub opcode: u8, // Instruction opcode is fetched here
    pub cycles: u8, // Counts how many cycles the instruction has remaining
    
    pub bus: Bus,

    pub system_clock_counter: u32,
}
    
impl CPU {
    pub fn new(cartridge: Cartridge) -> Self {
        CPU {
            accumulator: 0x00,
            x_register: 0x00,
            y_register: 0x00,
            stack_pointer: 0x00,
            program_counter: 0x0000,
            status: 0x00,

            fetched: 0x00,

            addr_abs: 0x0000,
            addr_rel: 0x0000,
            opcode: 0x00,
            cycles: 0x00,

            bus: Bus::new(cartridge),

            system_clock_counter: 0,
        } 
    }

    pub fn read(&mut self, addr: u16, _b_read_only: bool) -> u8 {
        self.bus.mem_read(addr)
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.bus.mem_write(addr, data);
    }

    pub fn clock(&mut self) {

        self.bus.ppu.clock();

        // CPU runs 1/3 as fast as PPU

        if self.system_clock_counter % 3 == 0 {
            if self.cycles == 0 {
                self.opcode = self.read(self.program_counter, false);
                self.program_counter += 1;

                let operate = &references::INSTRUCTION_LOOKUP[self.opcode as usize].operate;
                let addressing_mode = &references::INSTRUCTION_LOOKUP[self.opcode as usize].addrmode;

                self.cycles = references::INSTRUCTION_LOOKUP[self.opcode as usize].cycles;


                let additional_cycle1: u8 = match addressing_mode {
                    AddressingMode::IMP => self.imp(),
                    AddressingMode::IMM => self.imm(),
                    AddressingMode::ZP0 => self.zp0(),
                    AddressingMode::ZPX => self.zpx(),
                    AddressingMode::ZPY => self.zpy(),
                    AddressingMode::REL => self.rel(),
                    AddressingMode::ABS => self.abs(),
                    AddressingMode::ABX => self.abx(),
                    AddressingMode::ABY => self.aby(),
                    AddressingMode::IND => self.ind(),
                    AddressingMode::IZX => self.izx(),
                    AddressingMode::IZY => self.izy(),
                };

                let additional_cycle2: u8 = match operate {
                    Opcode::ADC => self.adc(),
                    Opcode::AND => self.and(),
                    Opcode::ASL => self.asl(),
                    Opcode::BCC => self.bcc(),
                    Opcode::BCS => self.bcs(),
                    Opcode::BEQ => self.beq(),
                    Opcode::BIT => self.bit(),
                    Opcode::BMI => self.bmi(),
                    Opcode::BNE => self.bne(),
                    Opcode::BPL => self.bpl(),
                    Opcode::BRK => self.brk(),
                    Opcode::BVC => self.bvc(),
                    Opcode::BVS => self.bvs(),
                    Opcode::CLC => self.clc(),
                    Opcode::CLD => self.cld(),
                    Opcode::CLI => self.cli(),
                    Opcode::CLV => self.clv(),
                    Opcode::CMP => self.cmp(),
                    Opcode::CPX => self.cpx(),
                    Opcode::CPY => self.cpy(),
                    Opcode::DEC => self.dec(),
                    Opcode::DEX => self.dex(),
                    Opcode::DEY => self.dey(),
                    Opcode::EOR => self.eor(),
                    Opcode::INC => self.inc(),
                    Opcode::INX => self.inx(),
                    Opcode::INY => self.iny(),
                    Opcode::JMP => self.jmp(),
                    Opcode::JSR => self.jsr(),
                    Opcode::LDA => self.lda(),
                    Opcode::LDX => self.ldx(),
                    Opcode::LDY => self.ldy(),
                    Opcode::LSR => self.lsr(),
                    Opcode::NOP => self.nop(),
Opcode::ORA => self.ora(),
Opcode::PHA => self.pha(),
                    Opcode::PHP => self.php(),
Opcode::PLA => self.pla(),
                    Opcode::PLP => self.plp(),
                    Opcode::ROL => self.rol(),
                    Opcode::ROR => self.ror(),
                    Opcode::RTI => self.rti(),
                    Opcode::RTS => self.rts(),
                    Opcode::SBC => self.sbc(),
                    Opcode::SEC => self.sec(),
                    Opcode::SED => self.sed(),
                    Opcode::SEI => self.sei(),
                    Opcode::STA => self.sta(),
                    Opcode::STX => self.stx(),
                    Opcode::STY => self.sty(),
                    Opcode::TAX => self.tax(),
                    Opcode::TAY => self.tay(),
                    Opcode::TSX => self.tsx(),
                    Opcode::TXA => self.txa(),
                    Opcode::TXS => self.txs(),
                    Opcode::TYA => self.tya(),
                    Opcode::XXX => self.xxx(),
                };


                self.cycles += additional_cycle1 & additional_cycle2;

                self.set_flag(StatusFlag::U, true);
            }

            self.cycles -= 1;
        }

        // When entering vblank, the PPU will set the NMI flag 
        if self.bus.ppu.nmi {
            self.bus.ppu.nmi = false;
            self.nmi();
        }

        self.system_clock_counter += 1;
    }
}

impl CPU {
    // Flags Functions
    fn set_flag(&mut self, flag: StatusFlag, value: bool) {
        if value {
            self.status |= flag as u8;
        } else {
            self.status &= !(flag as u8);
        }
    }

    fn get_flag(&self, flag: StatusFlag) -> u8 {
        if (self.status & (flag as u8)) > 0 {
            1
        } else {
            0
        }
    }

    // Addressing Modes
    // Returned integer is the additional number of cycles required for the Instruction

    fn imp(&mut self) -> u8{
        // Some instructions use the accumulator's value as operand
        self.fetched = self.accumulator;
        0
    }

    fn imm(&mut self) -> u8 {
        self.addr_abs = self.program_counter;
        self.program_counter += 1;
        0
    }

    fn zp0(&mut self) -> u8 {
        self.addr_abs = self.read(self.program_counter, false) as u16;
        self.program_counter += 1;
        self.addr_abs &= 0x00FF;
        0
    }

    fn zpx(&mut self) -> u8 {
        let base = self.read(self.program_counter, false);
        self.addr_abs = base.wrapping_add(self.x_register) as u16;
        self.program_counter += 1;
        self.addr_abs &= 0x00FF;
        0
    }

    fn zpy(&mut self) -> u8 {
        let base = self.read(self.program_counter, false);
        self.addr_abs = base.wrapping_add(self.y_register) as u16;
        self.program_counter += 1;
        self.addr_abs &= 0x00FF;
        0
    }

    fn abs(&mut self) -> u8 {
        //6502 stores memory address in little endian format
        let lo = self.read(self.program_counter, false) as u16;
        self.program_counter += 1;

        let hi = self.read(self.program_counter, false) as u16;
        self.program_counter += 1;

        self.addr_abs = (hi << 8) | lo;
        0
    }

    fn abx(&mut self) -> u8 {
        let lo = self.read(self.program_counter, false) as u16;
        self.program_counter += 1;

        let hi = self.read(self.program_counter, false) as u16;
        self.program_counter += 1;

        self.addr_abs = (hi << 8) | lo;
        self.addr_abs += self.x_register as u16;

        // If the addition of the offset causes a change in the high byte, an additional cycle is required
        if (self.addr_abs & 0xFF00) != (hi << 8) {
            1
        } else {
            0
        }
    }

    fn aby(&mut self) -> u8 {
        let lo = self.read(self.program_counter, false) as u16;
        self.program_counter += 1;

        let hi = self.read(self.program_counter, false) as u16;
        self.program_counter += 1;

        self.addr_abs = (hi << 8) | lo;
        self.addr_abs += self.y_register as u16;

        // If the addition of the offset causes a change in the high byte, an additional cycle is required
        if (self.addr_abs & 0xFF00) != (hi << 8) {
            1
        } else {
            0
        }
    }

    fn ind(&mut self) -> u8 {
        let ptr_lo = self.read(self.program_counter, false) as u16;
        self.program_counter += 1;

        let ptr_hi = self.read(self.program_counter, false) as u16;
        self.program_counter += 1;

        let ptr = (ptr_hi << 8) | ptr_lo;

        // 6502 bug where if the low byte of the supplied address is 0xFF, the high byte is fetched from the low byte of the supplied address
        // This is added for bug for bug compatibility
        if ptr_lo == 0x00FF {
            self.addr_abs = (self.read(ptr & 0xFF00, false) as u16) << 8 | self.read(ptr, false) as u16;
        } else {
            self.addr_abs = (self.read(ptr + 1, false) as u16) << 8 | self.read(ptr, false) as u16;
        }

        0
    }

    fn izx(&mut self) -> u8 {
        let t = self.read(self.program_counter, false) as u16;
        self.program_counter += 1;

        let lo = self.read((t + self.x_register as u16) & 0x00FF, false) as u16;
        let hi = self.read((t + self.x_register as u16 + 1) & 0x00FF, false) as u16;

        self.addr_abs = (hi << 8) | lo;
        0
    }

    fn izy(&mut self) -> u8 {
        let t = self.read(self.program_counter, false) as u16;
        self.program_counter += 1;

        let lo = self.read(t & 0x00FF, false) as u16;
        let hi = self.read((t + 1) & 0x00FF, false) as u16;

        self.addr_abs = (hi << 8) | lo;
        self.addr_abs += self.y_register as u16;

        // If the addition of the offset causes a change in the high byte, an additional cycle is required
        if (self.addr_abs & 0xFF00) != (hi << 8) {
            1
        } else {
            0
        }
    }

    fn rel(&mut self) -> u8 {
        self.addr_rel = self.read(self.program_counter, false) as u16;
        self.program_counter += 1;

        // two's complement to convert to signed integer
        if (self.addr_rel & 0x80) != 0 {
            self.addr_rel |= 0xFF00;
        }

        0
    }
    // fetches data from memory using the address mode
    fn fetch(&mut self) -> u8 {
        if references::INSTRUCTION_LOOKUP[self.opcode as usize].addrmode != AddressingMode::IMP {
            self.fetched = self.read(self.addr_abs, false);
        }

        self.fetched
    }

    // Instructions
    fn and(&mut self) -> u8 {
        self.fetch();
        self.accumulator &= self.fetched;
        self.set_flag(StatusFlag::Z, self.accumulator == 0x00);
        self.set_flag(StatusFlag::N, self.accumulator & 0x80 != 0);
        1
    }


    fn bcs(&mut self) -> u8 {
        if self.get_flag(StatusFlag::C) == 1 {
            self.cycles += 1;
            self.addr_abs = self.program_counter.wrapping_add(self.addr_rel);

            // If the branch crosses a page boundary, an additional cycle is required
            if (self.addr_abs & 0xFF00) != (self.program_counter & 0xFF00) {
                self.cycles += 1;
            }

            self.program_counter = self.addr_abs;
        }

        0
    }

    fn bcc(&mut self) -> u8 {
        if self.get_flag(StatusFlag::C) == 0{
            self.cycles += 1;
            self.addr_abs = self.program_counter.wrapping_add(self.addr_rel);

            // If the branch crosses a page boundary, an additional cycle is required
            if (self.addr_abs & 0xFF00) != (self.program_counter & 0xFF00) {
                self.cycles += 1;
            }

            self.program_counter = self.addr_abs;
        }

        0
    }

    fn beq(&mut self) -> u8 {
        if self.get_flag(StatusFlag::Z) == 1 {
            self.cycles += 1;
            self.addr_abs = self.program_counter.wrapping_add(self.addr_rel);

            // If the branch crosses a page boundary, an additional cycle is required
            if (self.addr_abs & 0xFF00) != (self.program_counter & 0xFF00) {
                self.cycles += 1;
            }

            self.program_counter = self.addr_abs;
        }

        0
    }

    fn bmi(&mut self) -> u8 {
        if self.get_flag(StatusFlag::N) == 1{
            self.cycles += 1;
            self.addr_abs = self.program_counter.wrapping_add(self.addr_rel);

            // If the branch crosses a page boundary, an additional cycle is required
            if (self.addr_abs & 0xFF00) != (self.program_counter & 0xFF00) {
                self.cycles += 1;
            }

            self.program_counter = self.addr_abs;
        }

        0
    }

    fn bne(&mut self) -> u8 {
        if self.get_flag(StatusFlag::Z) == 0 {
            self.cycles += 1;
            self.addr_abs = self.program_counter.wrapping_add(self.addr_rel);

            // If the branch crosses a page boundary, an additional cycle is required
            if (self.addr_abs & 0xFF00) != (self.program_counter & 0xFF00) {
                self.cycles += 1;
            }

            self.program_counter = self.addr_abs;
        }

        0
    }

    fn bpl(&mut self) -> u8 {
        if self.get_flag(StatusFlag::N) == 0{
            self.cycles += 1;
            self.addr_abs = self.program_counter.wrapping_add(self.addr_rel);

            // If the branch crosses a page boundary, an additional cycle is required
            if (self.addr_abs & 0xFF00) != (self.program_counter & 0xFF00) {
                self.cycles += 1;
            }

            self.program_counter = self.addr_abs;
        }

        0
    }

    fn bvc(&mut self) -> u8 {
        if self.get_flag(StatusFlag::V) == 0{
            self.cycles += 1;
            self.addr_abs = self.program_counter.wrapping_add(self.addr_rel);

            // If the branch crosses a page boundary, an additional cycle is required
            if (self.addr_abs & 0xFF00) != (self.program_counter & 0xFF00) {
                self.cycles += 1;
            }

            self.program_counter = self.addr_abs;
        }

        0
    }

    fn bvs(&mut self) -> u8 {
        if self.get_flag(StatusFlag::V) == 1{
            self.cycles += 1;
            self.addr_abs = self.program_counter.wrapping_add(self.addr_rel);

            // If the branch crosses a page boundary, an additional cycle is required
            if (self.addr_abs & 0xFF00) != (self.program_counter & 0xFF00) {
                self.cycles += 1;
            }

            self.program_counter = self.addr_abs;
        }

        0
    }

    fn jmp(&mut self) -> u8 {
        self.program_counter = self.addr_abs;
        0
    }

    fn jsr(&mut self) -> u8 {
        self.program_counter = self.program_counter.wrapping_sub(1);

        self.write(0x0100 + self.stack_pointer as u16, (self.program_counter >> 8) as u8);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
        self.write(0x0100 + self.stack_pointer as u16, self.program_counter as u8);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);

        self.program_counter = self.addr_abs;
        0
    }

    fn lda(&mut self) -> u8 {
        self.fetch();
        self.accumulator = self.fetched;

        self.set_flag(StatusFlag::Z, self.accumulator == 0x00);
        self.set_flag(StatusFlag::N, self.accumulator & 0x80 != 0);

        1
    }

    fn ldx(&mut self) -> u8 {
        self.fetch();
        self.x_register = self.fetched;

        self.set_flag(StatusFlag::Z, self.x_register == 0x00);
        self.set_flag(StatusFlag::N, self.x_register & 0x80 != 0);

        1
    }

    fn ldy(&mut self) -> u8 {
        self.fetch();
        self.y_register = self.fetched;

        self.set_flag(StatusFlag::Z, self.y_register == 0x00);
        self.set_flag(StatusFlag::N, self.y_register & 0x80 != 0);

        1
    }

    fn clc(&mut self) -> u8 {
        self.set_flag(StatusFlag::C, false);
        0
    }

    fn cld(&mut self) -> u8 {
        self.set_flag(StatusFlag::D, false);
        0
    }

    fn cli(&mut self) -> u8 {
        self.set_flag(StatusFlag::I, false);
        0
    }

    fn clv(&mut self) -> u8 {
        self.set_flag(StatusFlag::V, false);
        0
    }

    fn sec(&mut self) -> u8 {
        self.set_flag(StatusFlag::C, true);
        0
    }

    fn sed(&mut self) -> u8 {
        self.set_flag(StatusFlag::D, true);
        0
    }

    fn sei(&mut self) -> u8 {
        self.set_flag(StatusFlag::I, true);
        0
    }

    fn sta(&mut self) -> u8 {
        self.write(self.addr_abs, self.accumulator);
        0
    }

    fn stx(&mut self) -> u8 {
        self.write(self.addr_abs, self.x_register);
        0
    }

    fn sty(&mut self) -> u8 {
        self.write(self.addr_abs, self.y_register);
        0
    }

    fn tax(&mut self) -> u8 {
        self.x_register = self.accumulator;
        self.set_flag(StatusFlag::Z, self.x_register == 0x00);
        self.set_flag(StatusFlag::N, (self.x_register & 0x80) != 0);
        0
    }

    fn tay(&mut self) -> u8 {
        self.y_register = self.accumulator;
        self.set_flag(StatusFlag::Z, self.y_register == 0x00);
        self.set_flag(StatusFlag::N, (self.y_register & 0x80) != 0);
        0
    }

    fn tsx(&mut self) -> u8 {
        self.x_register = self.stack_pointer;
        self.set_flag(StatusFlag::Z, self.x_register == 0x00);
        self.set_flag(StatusFlag::N, (self.x_register & 0x80) != 0);
        0
    }

    fn txa(&mut self) -> u8 {
        self.accumulator = self.x_register;
        self.set_flag(StatusFlag::Z, self.accumulator == 0x00);
        self.set_flag(StatusFlag::N, (self.accumulator & 0x80) != 0);
        0
    }

    fn txs(&mut self) -> u8 {
        self.stack_pointer = self.x_register;
        0
    }

    fn tya(&mut self) -> u8 {
        self.accumulator = self.y_register;
        self.set_flag(StatusFlag::Z, self.accumulator == 0x00);
        self.set_flag(StatusFlag::N, (self.accumulator & 0x80) != 0);
        0
    }

    fn adc(&mut self) -> u8 {
        self.fetch();
        let temp: u16 = self.accumulator as u16 + self.fetched as u16 + self.get_flag(StatusFlag::C) as u16;
        self.set_flag(StatusFlag::C, temp > 255);
        self.set_flag(StatusFlag::Z, (temp & 0x00FF) == 0);
        self.set_flag(StatusFlag::N, (temp & 0x80) != 0);
        self.set_flag(StatusFlag::V, ((!(self.accumulator ^ self.fetched) & (self.accumulator ^ temp as u8)) & 0x80) != 0);
        self.accumulator = temp as u8;
        1
    }

    fn sbc(&mut self) -> u8 {
        self.fetch();
        let value: u16 = (self.fetched as u16) ^ 0x00FF;
        let temp: u16 = self.accumulator as u16 + value + self.get_flag(StatusFlag::C) as u16;
        self.set_flag(StatusFlag::C, (temp & 0xFF00) != 0);
        self.set_flag(StatusFlag::Z, (temp & 0x00FF) == 0);
        self.set_flag(StatusFlag::V, ((temp ^ self.accumulator as u16) & (temp ^ value) & 0x0080) != 0);
        self.set_flag(StatusFlag::N, (temp & 0x0080) != 0);
        self.accumulator = temp as u8;
        1
    }

    fn asl(&mut self) -> u8 {
        self.fetch();
        let temp: u16 = (self.fetched as u16) << 1;
        self.set_flag(StatusFlag::C, (temp & 0xFF00) > 0);
        self.set_flag(StatusFlag::Z, (temp & 0x00FF) == 0x00);
        self.set_flag(StatusFlag::N, (temp & 0x0080) != 0);
        if references::INSTRUCTION_LOOKUP[self.opcode as usize].addrmode == AddressingMode::IMP {
            self.accumulator = temp as u8;
        } else {
            self.write(self.addr_abs, temp as u8);
        }
        0
    }

    fn lsr(&mut self) -> u8 {
        self.fetch();
        let temp: u16 = (self.fetched as u16) >> 1;
        self.set_flag(StatusFlag::C, (temp & 0x0001) != 0);
        self.set_flag(StatusFlag::Z, (temp & 0x00FF) == 0x0000);
        self.set_flag(StatusFlag::N, (temp & 0x0080) != 0);
        if references::INSTRUCTION_LOOKUP[self.opcode as usize].addrmode == AddressingMode::IMP {
            self.accumulator = temp as u8;
        } else {
            self.write(self.addr_abs, temp as u8);
        }
        0
    }

    fn rol(&mut self) -> u8 {
        self.fetch();
        let temp: u16 = (self.fetched as u16) << 1 | self.get_flag(StatusFlag::C) as u16;
        self.set_flag(StatusFlag::C, (temp & 0xFF00) != 0);
        self.set_flag(StatusFlag::Z, (temp & 0x00FF) == 0x0000);
        self.set_flag(StatusFlag::N, (temp & 0x0080) != 0);
        if references::INSTRUCTION_LOOKUP[self.opcode as usize].addrmode == AddressingMode::IMP {
            self.accumulator = temp as u8;
        } else {
            self.write(self.addr_abs, temp as u8);
        }
        0
    }

    fn ror(&mut self) -> u8 {
        self.fetch();
        let temp: u16 = (self.get_flag(StatusFlag::C) as u16) << 7 | (self.fetched as u16) >> 1;
        self.set_flag(StatusFlag::C, (self.fetched & 0x01) != 0);
        self.set_flag(StatusFlag::Z, (temp & 0x00FF) == 0x0000);
        self.set_flag(StatusFlag::N, (temp & 0x0080) != 0);
        if references::INSTRUCTION_LOOKUP[self.opcode as usize].addrmode == AddressingMode::IMP {
            self.accumulator = temp as u8;
        } else {
            self.write(self.addr_abs, temp as u8);
        }
        0
    }

    fn cmp(&mut self) -> u8 {
        self.fetch();
        let temp = self.accumulator.wrapping_sub(self.fetched) as u16;
        self.set_flag(StatusFlag::C, self.accumulator >= self.fetched);
        self.set_flag(StatusFlag::Z, (temp & 0x00FF) == 0x0000);
        self.set_flag(StatusFlag::N, (temp & 0x0080) != 0);
        1
    }

    fn cpx(&mut self) -> u8 {
        self.fetch();
        let temp = self.x_register.wrapping_sub(self.fetched) as u16;
        self.set_flag(StatusFlag::C, self.x_register >= self.fetched);
        self.set_flag(StatusFlag::Z, (temp & 0x00FF) == 0x0000);
        self.set_flag(StatusFlag::N, (temp & 0x0080) != 0);
        0
    }

    fn cpy(&mut self) -> u8 {
        self.fetch();
        let temp = self.y_register.wrapping_sub(self.fetched) as u16;
        self.set_flag(StatusFlag::C, self.y_register >= self.fetched);
        self.set_flag(StatusFlag::Z, (temp & 0x00FF) == 0x0000);
        self.set_flag(StatusFlag::N, (temp & 0x0080) != 0);
        0
    }

    fn inc(&mut self) -> u8 {
        self.fetch();
        let temp: u16 = self.fetched as u16 + 1;
        self.write(self.addr_abs, temp as u8);
        self.set_flag(StatusFlag::Z, (temp & 0x00FF) == 0x0000);
        self.set_flag(StatusFlag::N, (temp & 0x0080) != 0);
        0
    }

    fn inx(&mut self) -> u8 {
        self.x_register = self.x_register.wrapping_add(1);
        self.set_flag(StatusFlag::Z, self.x_register == 0x00);
        self.set_flag(StatusFlag::N, (self.x_register & 0x80) != 0);
        0
    }

    fn iny(&mut self) -> u8 {
        self.y_register = self.y_register.wrapping_add(1);
        self.set_flag(StatusFlag::Z, self.y_register == 0x00);
        self.set_flag(StatusFlag::N, (self.y_register & 0x80) != 0);
        0
    }

    fn dec(&mut self) -> u8 {
        self.fetch();
        let temp = self.fetched.wrapping_sub(1) as u16;
        self.write(self.addr_abs, temp as u8);
        self.set_flag(StatusFlag::Z, (temp & 0x00FF) == 0x0000);
        self.set_flag(StatusFlag::N, (temp & 0x0080) != 0);
        0
    }

    fn dex(&mut self) -> u8 {
        self.x_register = self.x_register.wrapping_sub(1);
        self.set_flag(StatusFlag::Z, self.x_register == 0x00);
        self.set_flag(StatusFlag::N, (self.x_register & 0x80) != 0);
        0
    }

    fn dey(&mut self) -> u8 {
        self.y_register = self.y_register.wrapping_sub(1);
        self.set_flag(StatusFlag::Z, self.y_register == 0x00);
        self.set_flag(StatusFlag::N, (self.y_register & 0x80) != 0);
        0
    }

    fn eor(&mut self) -> u8 {
        self.fetch();
        self.accumulator = self.accumulator ^ self.fetched;
        self.set_flag(StatusFlag::Z, self.accumulator == 0x00);
        self.set_flag(StatusFlag::N, (self.accumulator & 0x80) != 0);
        1
    }

    fn ora(&mut self) -> u8 {
        self.fetch();
        self.accumulator = self.accumulator | self.fetched;
        self.set_flag(StatusFlag::Z, self.accumulator == 0x00);
        self.set_flag(StatusFlag::N, (self.accumulator & 0x80) != 0);
        1
    }

    fn bit(&mut self) -> u8 {
        self.fetch();
        let temp: u16 = self.accumulator as u16 & self.fetched as u16;
        self.set_flag(StatusFlag::Z, (temp & 0x00FF) == 0x0000);
        self.set_flag(StatusFlag::N, (self.fetched & (1 << 7)) != 0);
        self.set_flag(StatusFlag::V, (self.fetched & (1 << 6)) != 0);
        0
    }

    fn pha(&mut self) -> u8 {
        self.write(0x0100 + self.stack_pointer as u16, self.accumulator);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
        0
    }

    fn php(&mut self) -> u8 {
        self.write(0x0100 + self.stack_pointer as u16, self.status | StatusFlag::B as u8 | StatusFlag::U as u8);
        self.set_flag(StatusFlag::B, false);
        self.set_flag(StatusFlag::U, false);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
        0
    }

    fn pla(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.accumulator = self.read(0x0100 + self.stack_pointer as u16, false);
        self.set_flag(StatusFlag::Z, self.accumulator == 0x00);
        self.set_flag(StatusFlag::N, (self.accumulator & 0x80) != 0);
        0
    }

    fn plp(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.status = self.read(0x0100 + self.stack_pointer as u16, false);
        self.set_flag(StatusFlag::U, true);
        0
    }

    fn irq(&mut self) -> u8 {
        if self.get_flag(StatusFlag::I) == 0 {
            self.write(0x0100 + self.stack_pointer as u16, ((self.program_counter >> 8) & 0x00FF) as u8);
            self.stack_pointer = self.stack_pointer.wrapping_sub(1);
            self.write(0x0100 + self.stack_pointer as u16, (self.program_counter & 0x00FF) as u8);
            self.stack_pointer = self.stack_pointer.wrapping_sub(1);

            self.set_flag(StatusFlag::B, false);
            self.set_flag(StatusFlag::U, true);
            self.set_flag(StatusFlag::I, true);
            self.write(0x0100 + self.stack_pointer as u16, self.status);
            self.stack_pointer = self.stack_pointer.wrapping_sub(1);

            self.addr_abs = 0xFFFE;
            let lo = self.read(self.addr_abs, false) as u16;
            let hi = self.read(self.addr_abs + 1, false) as u16;
            self.program_counter = (hi << 8) | lo;

            self.cycles = 7;
        }

        0
    }

    fn nmi(&mut self) -> u8 {
        self.write(0x0100 + self.stack_pointer as u16, ((self.program_counter >> 8) & 0x00FF) as u8);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
        self.write(0x0100 + self.stack_pointer as u16, (self.program_counter & 0x00FF) as u8);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);

        self.set_flag(StatusFlag::B, false);
        self.set_flag(StatusFlag::U, true);
        self.set_flag(StatusFlag::I, true);
        self.write(0x0100 + self.stack_pointer as u16, self.status);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);

        self.addr_abs = 0xFFFA;
        let lo = self.read(self.addr_abs, false) as u16;
        let hi = self.read(self.addr_abs + 1, false) as u16;
        self.program_counter = (hi << 8) | lo;

        self.cycles = 8;
        0
    }

    fn rti(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.status = self.read(0x0100 + self.stack_pointer as u16, false);
        self.status &= !(StatusFlag::B as u8);
        self.status &= !(StatusFlag::U as u8);

        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let lo = self.read(0x0100 + self.stack_pointer as u16, false) as u16;
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let hi = self.read(0x0100 + self.stack_pointer as u16, false) as u16;
        self.program_counter = (hi << 8) | lo;

        0
    }

    fn rts(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let lo = self.read(0x0100 + self.stack_pointer as u16, false) as u16;
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let hi = self.read(0x0100 + self.stack_pointer as u16, false) as u16;
        self.program_counter = (hi << 8) | lo;

        self.program_counter = self.program_counter.wrapping_add(1);
        0
    }

    fn brk(&mut self) -> u8 {
        self.program_counter = self.program_counter.wrapping_add(1);

        self.set_flag(StatusFlag::I, true);
        self.write(0x0100 + self.stack_pointer as u16, ((self.program_counter >> 8) & 0x00FF) as u8);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
        self.write(0x0100 + self.stack_pointer as u16, (self.program_counter & 0x00FF) as u8);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);

        self.set_flag(StatusFlag::B, true);
        self.write(0x0100 + self.stack_pointer as u16, self.status);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
        self.set_flag(StatusFlag::B, false);

        self.addr_abs = 0xFFFE;
        let lo = self.read(self.addr_abs, false) as u16;
        let hi = self.read(self.addr_abs + 1, false) as u16;
        self.program_counter = (hi << 8) | lo;

        0
    }

    fn nop(&mut self) -> u8 {
        // nop is a special case because it doesn't do anything
        // it's just a placeholder for the CPU to do nothing
        // so we don't need to do anything here
        // but we do need to return the number of cycles
        match self.opcode {
            0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => 1,
            _ => 0,
        }
    }

    fn xxx(&mut self) -> u8 {
        0
    }
    
    pub fn reset(&mut self) {
        self.addr_abs = 0xFFFC;
        let lo = self.read(self.addr_abs, false) as u16;
        let hi = self.read(self.addr_abs + 1, false) as u16;
        self.program_counter = (hi << 8) | lo;
        
        self.accumulator = 0;
        self.x_register = 0;
        self.y_register = 0;
        self.stack_pointer = 0xFD;
        self.status = StatusFlag::U as u8 | StatusFlag::I as u8;

        self.addr_rel = 0x0000;
        self.addr_abs = 0x0000;
        self.fetched = 0x00;

        self.cycles = 8;
    }

    pub fn complete(&mut self) -> bool {
        self.cycles == 0
    }


}    
