use std::fmt::format;

fn main() {
    let path = "program";
    let program_image = std::fs::read(path).unwrap();
    // println!("{:x?}", &program_image);
    println!("bits 16");

    let mut cpu = Cpu {
        program_image,
        pc: 0,
    };

    loop {
        if cpu.is_finished() {
            break;
        }

        let op = cpu.fetch_next_byte();

        let bits = (
            (op >> 7) & 1 as u8,
            (op >> 6) & 1 as u8,
            (op >> 5) & 1 as u8,
            (op >> 4) & 1 as u8,
            (op >> 3) & 1 as u8,
            (op >> 2) & 1 as u8,
            (op >> 1) & 1 as u8,
            (op >> 0) & 1 as u8,
        );

        match bits {
            (1, 0, 0, 0, 1, 0, d, w) => {
                let b1 = cpu.fetch_next_byte();

                let mod_bits = ((b1 >> 7 & 1) as u8, (b1 >> 6 & 1) as u8);
                match mod_bits {
                    // Memory Mode
                    (0, 0) => {
                        let (reg, m) = byte_to_reg_rm(b1);
                        let r = RegisterFile::get_reg(reg, w);
                        let m = match m {
                            0b000 => "[bx+si]",
                            0b001 => "[bx+di]",
                            0b010 => "[bp+si]",
                            0b011 => "[bp+di]",
                            0b100 => "[si]",
                            0b101 => "[di]",
                            0b110 => {
                                let disp0 = cpu.fetch_next_byte();
                                let disp1 = cpu.fetch_next_byte();
                                todo!()
                            }
                            0b111 => "[bx]",
                            _ => unreachable!(),
                        }
                        .to_owned();
                        let (src, dest) = if d == 1 { (m, r) } else { (r, m) };
                        println!("mov {dest}, {src}");
                    }
                    // Memory Mode, 8bit displacement
                    (0, 1) => {
                        let (reg, m) = byte_to_reg_rm(b1);
                        let r = RegisterFile::get_reg(reg, w);
                        let disp = cpu.fetch_next_byte();
                        let m = match m {
                            0b000 => format!("[bx+si+{disp}]"),
                            0b001 => format!("[bx+di+{disp}]"),
                            0b010 => format!("[bp+si+{disp}]"),
                            0b011 => format!("[bp+di+{disp}]"),
                            0b100 => format!("[si+{disp}]"),
                            0b101 => format!("[di+{disp}]"),
                            0b110 => format!("[bp+{disp}]"),
                            0b111 => format!("[dx+{disp}]"),
                            _ => unreachable!(),
                        };

                        let (src, dest) = if d == 1 { (m, r) } else { (r, m) };
                        println!("mov {dest}, {src}");
                    }
                    // Memory Mode, 16bit displacement
                    (1, 0) => {
                        let (reg, m) = byte_to_reg_rm(b1);
                        let r = RegisterFile::get_reg(reg, w);
                        let disp0 = cpu.fetch_next_byte();
                        let disp1 = cpu.fetch_next_byte();
                        let disp: u16 = ((disp1 as u16) << 8) + disp0 as u16;
                        let m = match m {
                            0b000 => format!("[bx+si+{disp}]"),
                            0b001 => format!("[bx+di+{disp}]"),
                            0b010 => format!("[bp+si+{disp}]"),
                            0b011 => format!("[bp+di+{disp}]"),
                            0b100 => format!("[si+{disp}]"),
                            0b101 => format!("[di+{disp}]"),
                            0b110 => format!("[bp+{disp}]"),
                            0b111 => format!("[dx+{disp}]"),
                            _ => unreachable!(),
                        };
                        let (src, dest) = if d == 1 { (m, r) } else { (r, m) };
                        println!("mov {dest}, {src}");
                    }
                    // Register Mode
                    (1, 1) => {
                        let (reg1, reg0) = byte_to_reg_rm(b1);
                        let r0 = RegisterFile::get_reg(reg0, w);
                        let r1 = RegisterFile::get_reg(reg1, w);
                        let (src, dest) = if d == 1 { (r0, r1) } else { (r1, r0) };
                        println!("mov {dest}, {src}");
                    }
                    _ => unimplemented!(),
                }
            }
            // Immediate to register
            (1, 0, 1, 1, w, r2, r1, r0) => {
                let reg = (r2 << 2) + (r1 << 1) + r0;
                let r1 = RegisterFile::get_reg(reg, w);
                let data0 = cpu.fetch_next_byte();
                if w == 1 {
                    let data1 = cpu.fetch_next_byte();
                    let data: u16 = ((data1 as u16) << 8) + data0 as u16;
                    println!("mov {r1}, {data}");
                } else {
                    println!("mov {r1}, {data0}");
                }
            }
            _ => unimplemented!(),
        }
    }
}

fn byte_to_reg_rm(byte: u8) -> (u8, u8) {
    let reg = (byte & 0x38) >> 3;
    let rm = byte & 0x07;
    (reg, rm)
}

struct Cpu {
    program_image: Vec<u8>,
    pc: usize,
}

impl Cpu {
    fn fetch_next_byte(&mut self) -> u8 {
        let byte = self.program_image[self.pc];
        self.pc += 1;
        return byte;
    }

    fn is_finished(&self) -> bool {
        self.pc >= self.program_image.len()
    }
}

#[derive(Default)]
struct RegisterFile {}

impl RegisterFile {
    const AL: u8 = 0b00000000;
    const CL: u8 = 0b00000001;
    const DL: u8 = 0b00000010;
    const BL: u8 = 0b00000011;
    const AH: u8 = 0b00000100;
    const CH: u8 = 0b00000101;
    const DH: u8 = 0b00000110;
    const BH: u8 = 0b00000111;

    const AX: u8 = 0b00000000;
    const CX: u8 = 0b00000001;
    const DX: u8 = 0b00000010;
    const BX: u8 = 0b00000011;
    const SP: u8 = 0b00000100;
    const BP: u8 = 0b00000101;
    const SI: u8 = 0b00000110;
    const DI: u8 = 0b00000111;

    fn get_reg(encoding: u8, w: u8) -> String {
        match w {
            0 => match encoding {
                Self::AL => "al".into(),
                Self::CL => "cl".into(),
                Self::DL => "dl".into(),
                Self::BL => "bl".into(),
                Self::AH => "ah".into(),
                Self::CH => "ch".into(),
                Self::DH => "dh".into(),
                Self::BH => "bh".into(),
                _ => unreachable!(),
            },
            1 => match encoding {
                Self::AX => "ax".into(),
                Self::CX => "cx".into(),
                Self::DX => "dx".into(),
                Self::BX => "bx".into(),
                Self::SP => "sp".into(),
                Self::BP => "bp".into(),
                Self::SI => "si".into(),
                Self::DI => "di".into(),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}
