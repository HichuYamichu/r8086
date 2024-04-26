use std::fmt::format;

fn main() {
    let path = "program";
    let program_image = std::fs::read(path).unwrap();
    // println!("{:x?}", &program_image);
    println!("bits 16");

    let mut program = Program {
        program_image,
        pc: 0,
    };

    loop {
        if program.is_finished() {
            break;
        }

        let op = program.fetch_next_byte();
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
            // Register/memory to/from register
            (1, 0, 0, 0, 1, 0, d, w) => {
                let b1 = program.fetch_next_byte();

                let mod_bits = b1 >> 6;
                match mod_bits {
                    // Memory Mode
                    0b00 => {
                        let (reg, m) = byte_to_reg_rm(b1);
                        let r = decode_register(reg, w);
                        let m = decode_address(&mut program, m);
                        let (src, dest) = to_src_dest(d, m, r);
                        println!("mov {dest}, {src}");
                    }
                    // Memory Mode, 8bit displacement
                    0b01 => {
                        let (reg, m) = byte_to_reg_rm(b1);
                        let r = decode_register(reg, w);
                        let disp = program.fetch_next_byte() as i8;
                        let m = decode_address_disp8(m, disp);
                        let (src, dest) = to_src_dest(d, m, r);
                        println!("mov {dest}, {src}");
                    }
                    // Memory Mode, 16bit displacement
                    0b10 => {
                        let (reg, m) = byte_to_reg_rm(b1);
                        let r = decode_register(reg, w);
                        let disp0 = program.fetch_next_byte();
                        let disp1 = program.fetch_next_byte();
                        let disp: i16 = ((disp1 as i16) << 8) + disp0 as i16;
                        let m = decode_address_disp16(m, disp);
                        let (src, dest) = to_src_dest(d, m, r);
                        println!("mov {dest}, {src}");
                    }
                    // Register Mode
                    0b11 => {
                        let (reg1, reg0) = byte_to_reg_rm(b1);
                        let r0 = decode_register(reg0, w);
                        let r1 = decode_register(reg1, w);
                        let (src, dest) = to_src_dest(d, r0, r1);
                        println!("mov {dest}, {src}");
                    }
                    _ => unreachable!(),
                }
            }
            // Immediate to register/memory
            (1, 1, 0, 0, 0, 1, 1, w) => {
                let b1 = program.fetch_next_byte();
                let (_, rm) = byte_to_reg_rm(b1);
                let mod_bits = b1 >> 6;
                let target = match mod_bits {
                    // Memory Mode
                    0b00 => {
                        let (_, m) = byte_to_reg_rm(b1);
                        let m = decode_address(&mut program, m);
                        m
                    }
                    // Memory Mode, 8bit displacement
                    0b01 => {
                        let (_, m) = byte_to_reg_rm(b1);
                        let disp = program.fetch_next_byte() as i8;
                        let m = decode_address_disp8(m, disp);
                        m
                    }
                    // Memory Mode, 16bit displacement
                    0b10 => {
                        let (_, m) = byte_to_reg_rm(b1);
                        let disp0 = program.fetch_next_byte();
                        let disp1 = program.fetch_next_byte();
                        let disp: i16 = ((disp1 as i16) << 8) + disp0 as i16;
                        let m = decode_address_disp16(m, disp);
                        m
                    }
                    // Register Mode
                    0b11 => {
                        let (_, r) = byte_to_reg_rm(b1);
                        let r = decode_register(r, w);
                        r
                    }
                    _ => unreachable!(),
                };
                if w == 1 {
                    let data0 = program.fetch_next_byte();
                    let data1 = program.fetch_next_byte();
                    let imm: u16 = ((data1 as u16) << 8) + data0 as u16;
                    println!("mov {target}, word {imm}");
                } else {
                    let imm = program.fetch_next_byte();
                    println!("mov {target}, byte {imm}");
                }
            }
            // Immediate to register
            (1, 0, 1, 1, w, r2, r1, r0) => {
                let reg = (r2 << 2) + (r1 << 1) + r0;
                let r1 = decode_register(reg, w);
                let data0 = program.fetch_next_byte();
                if w == 1 {
                    let data1 = program.fetch_next_byte();
                    let data: u16 = ((data1 as u16) << 8) + data0 as u16;
                    println!("mov {r1}, {data}");
                } else {
                    println!("mov {r1}, {data0}");
                }
            }
            // Memory to accumulator
            (1, 0, 1, 0, 0, 0, 0, w) => {
                let data0 = program.fetch_next_byte();
                if w == 1 {
                    let data1 = program.fetch_next_byte();
                    let data: u16 = ((data1 as u16) << 8) + data0 as u16;
                    println!("mov ax, [{data}]");
                } else {
                    println!("mov ax, [{data0}]");
                }
            }
            // Accumulator to memory
            (1, 0, 1, 0, 0, 0, 1, w) => {
                let data0 = program.fetch_next_byte();
                if w == 1 {
                    let data1 = program.fetch_next_byte();
                    let data: u16 = ((data1 as u16) << 8) + data0 as u16;
                    println!("mov [{data}], ax");
                } else {
                    println!("mov [{data0}], ax");
                }
            }
            _ => unimplemented!(),
        }
    }
}

fn to_src_dest(d: u8, r0: String, r1: String) -> (String, String) {
    if d == 1 {
        (r0, r1)
    } else {
        (r1, r0)
    }
}

fn byte_to_reg_rm(byte: u8) -> (u8, u8) {
    let reg = (byte & 0x38) >> 3;
    let rm = byte & 0x07;
    (reg, rm)
}

struct Program {
    program_image: Vec<u8>,
    pc: usize,
}

impl Program {
    fn fetch_next_byte(&mut self) -> u8 {
        let byte = self.program_image[self.pc];
        self.pc += 1;
        return byte;
    }

    fn is_finished(&self) -> bool {
        self.pc >= self.program_image.len()
    }
}

fn decode_register(encoding: u8, wide: u8) -> String {
    match wide {
        0 => match encoding {
            0b000 => "al".into(),
            0b001 => "cl".into(),
            0b010 => "dl".into(),
            0b011 => "bl".into(),
            0b100 => "ah".into(),
            0b101 => "ch".into(),
            0b110 => "dh".into(),
            0b111 => "bh".into(),
            _ => unreachable!(),
        },
        1 => match encoding {
            0b000 => "ax".into(),
            0b001 => "cx".into(),
            0b010 => "dx".into(),
            0b011 => "bx".into(),
            0b100 => "sp".into(),
            0b101 => "bp".into(),
            0b110 => "si".into(),
            0b111 => "di".into(),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

fn decode_address(program: &mut Program, encoding: u8) -> String {
    let address = match encoding {
        0b000 => "[bx+si]".into(),
        0b001 => "[bx+di]".into(),
        0b010 => "[bp+si]".into(),
        0b011 => "[bp+di]".into(),
        0b100 => "[si]".into(),
        0b101 => "[di]".into(),
        0b110 => {
            let disp0 = program.fetch_next_byte();
            let disp1 = program.fetch_next_byte();
            let disp: u16 = ((disp1 as u16) << 8) + disp0 as u16;
            format!("[{disp}]")
        }
        0b111 => "[bx]".into(),
        _ => unreachable!(),
    };
    address
}

fn decode_address_disp8(encoding: u8, disp: i8) -> String {
    let address = match encoding {
        0b000 => format!("[bx+si{:+}]", disp),
        0b001 => format!("[bx+di{:+}]", disp),
        0b010 => format!("[bp+si{:+}]", disp),
        0b011 => format!("[bp+di{:+}]", disp),
        0b100 => format!("[si{:+}]", disp),
        0b101 => format!("[di{:+}]", disp),
        0b110 => format!("[bp{:+}]", disp),
        0b111 => format!("[bx{:+}]", disp),
        _ => unreachable!(),
    };
    address
}

fn decode_address_disp16(encoding: u8, disp: i16) -> String {
    let address = match encoding {
        0b000 => format!("[bx+si{:+}]", disp),
        0b001 => format!("[bx+di{:+}]", disp),
        0b010 => format!("[bp+si{:+}]", disp),
        0b011 => format!("[bp+di{:+}]", disp),
        0b100 => format!("[si{:+}]", disp),
        0b101 => format!("[di{:+}]", disp),
        0b110 => format!("[bp{:+}]", disp),
        0b111 => format!("[bx{:+}]", disp),
        _ => unreachable!(),
    };
    address
}
