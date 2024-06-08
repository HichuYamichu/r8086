use crate::*;

pub fn decode_instruction(bytes: &[u8]) -> Instruction {
    let bits = (
        (bytes[0] >> 7) & 1 as u8,
        (bytes[0] >> 6) & 1 as u8,
        (bytes[0] >> 5) & 1 as u8,
        (bytes[0] >> 4) & 1 as u8,
        (bytes[0] >> 3) & 1 as u8,
        (bytes[0] >> 2) & 1 as u8,
        (bytes[0] >> 1) & 1 as u8,
        (bytes[0] >> 0) & 1 as u8,
    );

    match bits {
        // MOV | Register/memory to/from register
        (1, 0, 0, 0, 1, 0, d, w) => {
            let mod_bits = bytes[1] >> 6;
            let reg = (bytes[1] & 0x38) >> 3;
            let rm = bytes[1] & 0x07;
            let disp_lo = bytes[2];
            let disp_hi = bytes[3];

            let reg0 = decode_register(reg, w);
            let reg1 = decode_register(rm, w);

            let (mem, bytes_read) = decode_address(rm, w, &bytes[2..4]);
            let mem_disp8 = decode_address_disp8(rm, w, disp_lo as i8);
            let disp16: i16 = ((disp_hi as i16) << 8) + disp_lo as i16;
            let mem_disp16 = decode_address_disp16(rm, w, disp16);

            let (src, dest, len) = match mod_bits {
                // Memory Mode
                0b00 if d == 1 => (mem, reg0, 2 + bytes_read),
                0b00 if d == 0 => (reg0, mem, 2 + bytes_read),
                // Memory Mode, 8bit displacement
                0b01 if d == 1 => (mem_disp8, reg0, 3),
                0b01 if d == 0 => (reg0, mem_disp8, 3),
                // Memory Mode, 16bit displacement
                0b10 if d == 1 => (mem_disp16, reg0, 4),
                0b10 if d == 0 => (reg0, mem_disp16, 4),
                // Register Mode
                0b11 if d == 1 => (reg1, reg0, 2),
                0b11 if d == 0 => (reg0, reg1, 2),
                _ => unreachable!(),
            };

            let instruction = Instruction {
                op: Op::Mov,
                operands: [Some(dest), Some(src)],
                length: len,
            };
            instruction
        }
        // MOV | Immediate to register/memory
        (1, 1, 0, 0, 0, 1, 1, w) => {
            let mod_bits = bytes[1] >> 6;
            let reg = (bytes[1] & 0x38) >> 3;
            let rm = bytes[1] & 0x07;
            let disp_lo = bytes[2];
            let disp_hi = bytes[3];

            let reg0 = decode_register(reg, w);

            let (mem, bytes_read) = decode_address(rm, w, &bytes[2..4]);
            let mem_disp8 = decode_address_disp8(rm, w, disp_lo as i8);
            let disp16: i16 = ((disp_hi as i16) << 8) + disp_lo as i16;
            let mem_disp16 = decode_address_disp16(rm, w, disp16);

            let (dest, data_lo, data_hi, len) = match mod_bits {
                // Memory Mode
                0b00 => {
                    let data_lo = bytes[2 + bytes_read as usize];
                    let data_hi = bytes[3 + bytes_read as usize];
                    (mem, data_lo, data_hi, 3 + bytes_read)
                }
                // Memory Mode, 8bit displacement
                0b01 => {
                    let data_lo = bytes[3];
                    let data_hi = bytes[4];
                    (mem_disp8, data_lo, data_hi, 4)
                }
                // Memory Mode, 16bit displacement
                0b10 => {
                    let data_lo = bytes[4];
                    let data_hi = bytes[5];
                    (mem_disp16, data_lo, data_hi, 5)
                }
                // Register Mode
                0b11 => {
                    let data_lo = bytes[2];
                    let data_hi = bytes[3];
                    (reg0, data_lo, data_hi, 3)
                }
                _ => unreachable!(),
            };

            let imm = if w == 0 {
                Operand::Immediate(Immediate::Bit8(data_lo as _))
            } else {
                let imm: u16 = ((data_hi as u16) << 8) + data_lo as u16;
                Operand::Immediate(Immediate::Bit16(imm))
            };

            let instruction = Instruction {
                op: Op::Mov,
                operands: [Some(dest), Some(imm)],
                length: len + w,
            };

            instruction
        }
        // MOV | Immediate to register
        (1, 0, 1, 1, w, r2, r1, r0) => {
            let reg = (r2 << 2) + (r1 << 1) + r0;
            let reg0 = decode_register(reg, w);
            let data_lo = bytes[1];
            let data_hi = bytes[2];
            let imm = if w == 0 {
                Operand::Immediate(Immediate::Bit8(data_lo as _))
            } else {
                let imm: u16 = ((data_hi as u16) << 8) + data_lo as u16;
                Operand::Immediate(Immediate::Bit16(imm))
            };

            let instruction = Instruction {
                op: Op::Mov,
                operands: [Some(reg0), Some(imm)],
                length: 2 + w,
            };

            instruction
        }
        // MOV | Memory to accumulator
        (1, 0, 1, 0, 0, 0, 0, w) => {
            let addr_lo = bytes[1];
            let addr_hi = bytes[2];

            let mem = if w == 0 {
                Operand::Memory(MemoryOperand {
                    kind: MemoryOperandKind::Direct(MemoryDirect::DirectAddress(addr_lo as _)),
                    size: MemoryOperandSize::Byte,
                })
            } else {
                let addr = ((addr_hi as u16) << 8) + addr_lo as u16;
                Operand::Memory(MemoryOperand {
                    kind: MemoryOperandKind::Direct(MemoryDirect::DirectAddress(addr)),
                    size: MemoryOperandSize::Word,
                })
            };

            let ax = Operand::Register(Register::AX);
            let instruction = Instruction {
                op: Op::Mov,
                operands: [Some(ax), Some(mem)],
                length: 2 + w,
            };

            instruction
        }
        // MOV | Accumulator to memory
        (1, 0, 1, 0, 0, 0, 1, w) => {
            let addr_lo = bytes[1];
            let addr_hi = bytes[2];

            let mem = if w == 0 {
                Operand::Memory(MemoryOperand {
                    kind: MemoryOperandKind::Direct(MemoryDirect::DirectAddress(addr_lo as _)),
                    size: MemoryOperandSize::Byte,
                })
            } else {
                let addr = ((addr_hi as u16) << 8) + addr_lo as u16;
                Operand::Memory(MemoryOperand {
                    kind: MemoryOperandKind::Direct(MemoryDirect::DirectAddress(addr)),
                    size: MemoryOperandSize::Word,
                })
            };

            let ax = Operand::Register(Register::AX);
            let instruction = Instruction {
                op: Op::Mov,
                operands: [Some(mem), Some(ax)],
                length: 2 + w,
            };

            instruction
        }
        // ADD | reg/memory with register to either
        (0, 0, 0, 0, 0, 0, d, w) => {
            let mod_bits = bytes[1] >> 6;
            let reg = (bytes[1] & 0x38) >> 3;
            let rm = bytes[1] & 0x07;
            let disp_lo = bytes[2];
            let disp_hi = bytes[3];

            let reg0 = decode_register(reg, w);
            let reg1 = decode_register(rm, w);

            let (mem, bytes_read) = decode_address(rm, w, &bytes[2..4]);
            let mem_disp8 = decode_address_disp8(rm, w, disp_lo as i8);
            let disp16: i16 = ((disp_hi as i16) << 8) + disp_lo as i16;
            let mem_disp16 = decode_address_disp16(rm, w, disp16);

            let (src, dest, len) = match mod_bits {
                // Memory Mode
                0b00 if d == 1 => (mem, reg0, 2 + bytes_read),
                0b00 if d == 0 => (reg0, mem, 2 + bytes_read),
                // Memory Mode, 8bit displacement
                0b01 if d == 1 => (mem_disp8, reg0, 3),
                0b01 if d == 0 => (reg0, mem_disp8, 3),
                // Memory Mode, 16bit displacement
                0b10 if d == 1 => (mem_disp16, reg0, 4),
                0b10 if d == 0 => (reg0, mem_disp16, 4),
                // Register Mode
                0b11 if d == 1 => (reg1, reg0, 2),
                0b11 if d == 0 => (reg0, reg1, 2),
                _ => unreachable!(),
            };

            let instruction = Instruction {
                op: Op::Add,
                operands: [Some(dest), Some(src)],
                length: len,
            };
            instruction
        }
        // ADD/SUB/CMP | immediate to register/memory
        (1, 0, 0, 0, 0, 0, s, w) => {
            let mod_bits = bytes[1] >> 6;
            let rm = bytes[1] & 0x07;
            let disp_lo = bytes[2];
            let disp_hi = bytes[3];

            let reg0 = decode_register(rm, w);

            let (mem, bytes_read) = decode_address(rm, w, &bytes[2..4]);
            let mem_disp8 = decode_address_disp8(rm, w, disp_lo as i8);
            let disp16: i16 = ((disp_hi as i16) << 8) + disp_lo as i16;
            let mem_disp16 = decode_address_disp16(rm, w, disp16);

            let (dest, src, len) = match mod_bits {
                // Memory Mode
                0b00 if w == 1 && s == 0 => {
                    let data_lo = bytes[2 + bytes_read as usize];
                    let data_hi = bytes[3 + bytes_read as usize];
                    let data: u16 = ((data_hi as u16) << 8) + data_lo as u16;
                    let imm = Operand::Immediate(Immediate::Bit16(data));
                    (mem, imm, 4 + bytes_read)
                }
                0b00 => {
                    let imm = Operand::Immediate(Immediate::Bit8(bytes[2 + bytes_read as usize]));
                    (mem, imm, 3 + bytes_read)
                }
                // Memory Mode, 8bit displacement
                0b01 if w == 1 && s == 0 => {
                    let data_lo = bytes[3];
                    let data_hi = bytes[4];
                    let data: u16 = ((data_hi as u16) << 8) + data_lo as u16;
                    let imm = Operand::Immediate(Immediate::Bit16(data));
                    (mem_disp8, imm, 5)
                }
                0b01 => {
                    let imm = Operand::Immediate(Immediate::Bit8(bytes[3]));
                    (mem_disp8, imm, 4)
                }
                // Memory Mode, 16bit displacement
                0b10 if w == 1 && s == 0 => {
                    let data_lo = bytes[4];
                    let data_hi = bytes[5];
                    let data: u16 = ((data_hi as u16) << 8) + data_lo as u16;
                    let imm = Operand::Immediate(Immediate::Bit16(data));
                    (mem_disp16, imm, 6)
                }
                0b10 => {
                    let imm = Operand::Immediate(Immediate::Bit8(bytes[4]));
                    (mem_disp16, imm, 5)
                }
                // Register Mode
                0b11 if w == 1 && s == 0 => {
                    let data_lo = bytes[2];
                    let data_hi = bytes[3];
                    let data: u16 = ((data_hi as u16) << 8) + data_lo as u16;
                    let imm = Operand::Immediate(Immediate::Bit16(data));
                    (reg0, imm, 4)
                }
                0b11 => {
                    let imm = Operand::Immediate(Immediate::Bit8(bytes[2]));
                    (reg0, imm, 3)
                }
                _ => unreachable!(),
            };
            let op_code = bytes[1] & 0b00111000;
            let op = match op_code {
                0b00000000 => Op::Add,
                0b00101000 => Op::Sub,
                0b00111000 => Op::Cmp,
                _ => unreachable!(),
            };
            let instruction = Instruction {
                op: op,
                operands: [Some(dest), Some(src)],
                length: len,
            };

            instruction
        }
        // ADD | immediate to accumulator
        (0, 0, 0, 0, 0, 1, 0, w) => {
            let data_lo = bytes[1];
            let data_hi = bytes[2];

            let (reg, imm) = if w == 0 {
                let al = Operand::Register(Register::AL);
                let operand = Operand::Immediate(Immediate::Bit8(data_lo));
                (al, operand)
            } else {
                let ax = Operand::Register(Register::AX);
                let data = ((data_hi as u16) << 8) + data_lo as u16;
                let operand = Operand::Immediate(Immediate::Bit16(data));
                (ax, operand)
            };

            let instruction = Instruction {
                op: Op::Add,
                operands: [Some(reg), Some(imm)],
                length: 2 + w,
            };

            instruction
        }
        // SUB | reg/memory and register to either
        (0, 0, 1, 0, 1, 0, d, w) => {
            let mod_bits = bytes[1] >> 6;
            let reg = (bytes[1] & 0x38) >> 3;
            let rm = bytes[1] & 0x07;
            let disp_lo = bytes[2];
            let disp_hi = bytes[3];

            let reg0 = decode_register(reg, w);
            let reg1 = decode_register(rm, w);

            let (mem, bytes_read) = decode_address(rm, w, &bytes[2..4]);
            let mem_disp8 = decode_address_disp8(rm, w, disp_lo as i8);
            let disp16: i16 = ((disp_hi as i16) << 8) + disp_lo as i16;
            let mem_disp16 = decode_address_disp16(rm, w, disp16);

            let (src, dest, len) = match mod_bits {
                // Memory Mode
                0b00 if d == 1 => (mem, reg0, 2 + bytes_read),
                0b00 if d == 0 => (reg0, mem, 2 + bytes_read),
                // Memory Mode, 8bit displacement
                0b01 if d == 1 => (mem_disp8, reg0, 3),
                0b01 if d == 0 => (reg0, mem_disp8, 3),
                // Memory Mode, 16bit displacement
                0b10 if d == 1 => (mem_disp16, reg0, 4),
                0b10 if d == 0 => (reg0, mem_disp16, 4),
                // Register Mode
                0b11 if d == 1 => (reg1, reg0, 2),
                0b11 if d == 0 => (reg0, reg1, 2),
                _ => unreachable!(),
            };

            let instruction = Instruction {
                op: Op::Sub,
                operands: [Some(dest), Some(src)],
                length: len,
            };
            instruction
        }
        // SUB | immediate from accumulator
        (0, 0, 1, 0, 1, 1, 0, w) => {
            let data_lo = bytes[1];
            let data_hi = bytes[2];

            let (reg, imm) = if w == 0 {
                let al = Operand::Register(Register::AL);
                let operand = Operand::Immediate(Immediate::Bit8(data_lo));
                (al, operand)
            } else {
                let ax = Operand::Register(Register::AX);
                let data = ((data_hi as u16) << 8) + data_lo as u16;
                let operand = Operand::Immediate(Immediate::Bit16(data));
                (ax, operand)
            };

            let instruction = Instruction {
                op: Op::Sub,
                operands: [Some(reg), Some(imm)],
                length: 2 + w,
            };

            instruction
        }
        // CMP | register/memory with register
        (0, 0, 1, 1, 1, 0, d, w) => {
            let mod_bits = bytes[1] >> 6;
            let reg = (bytes[1] & 0x38) >> 3;
            let rm = bytes[1] & 0x07;
            let disp_lo = bytes[2];
            let disp_hi = bytes[3];

            let reg0 = decode_register(reg, w);
            let reg1 = decode_register(rm, w);

            let (mem, bytes_read) = decode_address(rm, w, &bytes[2..4]);
            let mem_disp8 = decode_address_disp8(rm, w, disp_lo as i8);
            let disp16: i16 = ((disp_hi as i16) << 8) + disp_lo as i16;
            let mem_disp16 = decode_address_disp16(rm, w, disp16);

            let (src, dest, len) = match mod_bits {
                // Memory Mode
                0b00 if d == 1 => (mem, reg0, 2 + bytes_read),
                0b00 if d == 0 => (reg0, mem, 2 + bytes_read),
                // Memory Mode, 8bit displacement
                0b01 if d == 1 => (mem_disp8, reg0, 3),
                0b01 if d == 0 => (reg0, mem_disp8, 3),
                // Memory Mode, 16bit displacement
                0b10 if d == 1 => (mem_disp16, reg0, 4),
                0b10 if d == 0 => (reg0, mem_disp16, 4),
                // Register Mode
                0b11 if d == 1 => (reg1, reg0, 2),
                0b11 if d == 0 => (reg0, reg1, 2),
                _ => unreachable!(),
            };

            let instruction = Instruction {
                op: Op::Cmp,
                operands: [Some(dest), Some(src)],
                length: len,
            };
            instruction
        }
        // CMP | immediate to accumulator
        (0, 0, 1, 1, 1, 1, 0, w) => {
            let data_lo = bytes[1];
            let data_hi = bytes[2];

            let (reg, imm) = if w == 0 {
                let al = Operand::Register(Register::AL);
                let operand = Operand::Immediate(Immediate::Bit8(data_lo));
                (al, operand)
            } else {
                let ax = Operand::Register(Register::AX);
                let data = ((data_hi as u16) << 8) + data_lo as u16;
                let operand = Operand::Immediate(Immediate::Bit16(data));
                (ax, operand)
            };

            let instruction = Instruction {
                op: Op::Cmp,
                operands: [Some(reg), Some(imm)],
                length: 2 + w,
            };

            instruction
        }
        // JE
        (0, 1, 1, 1, 0, 1, 0, 0) => {
            let ip_inc = Operand::Immediate(Immediate::Bit8(bytes[1]));
            let instruction = Instruction {
                op: Op::Je,
                operands: [Some(ip_inc), None],
                length: 2,
            };

            instruction
        }
        // JL
        (0, 1, 1, 1, 1, 1, 0, 0) => {
            let ip_inc = Operand::Immediate(Immediate::Bit8(bytes[1]));
            let instruction = Instruction {
                op: Op::Jl,
                operands: [Some(ip_inc), None],
                length: 2,
            };

            instruction
        }
        // JLE
        (0, 1, 1, 1, 1, 1, 1, 0) => {
            let ip_inc = Operand::Immediate(Immediate::Bit8(bytes[1]));
            let instruction = Instruction {
                op: Op::Jle,
                operands: [Some(ip_inc), None],
                length: 2,
            };

            instruction
        }
        // JB
        (0, 1, 1, 1, 0, 0, 1, 0) => {
            let ip_inc = Operand::Immediate(Immediate::Bit8(bytes[1]));
            let instruction = Instruction {
                op: Op::Jb,
                operands: [Some(ip_inc), None],
                length: 2,
            };

            instruction
        }
        // JBE
        (0, 1, 1, 1, 0, 1, 1, 0) => {
            let ip_inc = Operand::Immediate(Immediate::Bit8(bytes[1]));
            let instruction = Instruction {
                op: Op::Jbe,
                operands: [Some(ip_inc), None],
                length: 2,
            };

            instruction
        }
        // JP
        (0, 1, 1, 1, 1, 0, 1, 0) => {
            let ip_inc = Operand::Immediate(Immediate::Bit8(bytes[1]));
            let instruction = Instruction {
                op: Op::Jp,
                operands: [Some(ip_inc), None],
                length: 2,
            };

            instruction
        }
        // JO
        (0, 1, 1, 1, 0, 0, 0, 0) => {
            let ip_inc = Operand::Immediate(Immediate::Bit8(bytes[1]));
            let instruction = Instruction {
                op: Op::Jo,
                operands: [Some(ip_inc), None],
                length: 2,
            };

            instruction
        }
        // JS
        (0, 1, 1, 1, 1, 0, 0, 0) => {
            let ip_inc = Operand::Immediate(Immediate::Bit8(bytes[1]));
            let instruction = Instruction {
                op: Op::Js,
                operands: [Some(ip_inc), None],
                length: 2,
            };

            instruction
        }
        // JNE/JNZ
        (0, 1, 1, 1, 0, 1, 0, 1) => {
            let ip_inc = Operand::Immediate(Immediate::Bit8(bytes[1]));
            let instruction = Instruction {
                op: Op::Jne,
                operands: [Some(ip_inc), None],
                length: 2,
            };

            instruction
        }
        // JNL
        (0, 1, 1, 1, 1, 1, 0, 1) => {
            let ip_inc = Operand::Immediate(Immediate::Bit8(bytes[1]));
            let instruction = Instruction {
                op: Op::Jnl,
                operands: [Some(ip_inc), None],
                length: 2,
            };

            instruction
        }
        // JG
        (0, 1, 1, 1, 1, 1, 1, 1) => {
            let ip_inc = Operand::Immediate(Immediate::Bit8(bytes[1]));
            let instruction = Instruction {
                op: Op::Jg,
                operands: [Some(ip_inc), None],
                length: 2,
            };

            instruction
        }
        // JNB
        (0, 1, 1, 1, 0, 0, 1, 1) => {
            let ip_inc = Operand::Immediate(Immediate::Bit8(bytes[1]));
            let instruction = Instruction {
                op: Op::Jnb,
                operands: [Some(ip_inc), None],
                length: 2,
            };

            instruction
        }
        // JA
        (0, 1, 1, 1, 0, 1, 1, 1) => {
            let ip_inc = Operand::Immediate(Immediate::Bit8(bytes[1]));
            let instruction = Instruction {
                op: Op::Ja,
                operands: [Some(ip_inc), None],
                length: 2,
            };

            instruction
        }
        // JNP
        (0, 1, 1, 1, 1, 0, 1, 1) => {
            let ip_inc = Operand::Immediate(Immediate::Bit8(bytes[1]));
            let instruction = Instruction {
                op: Op::Jnp,
                operands: [Some(ip_inc), None],
                length: 2,
            };

            instruction
        }
        // JNO
        (0, 1, 1, 1, 0, 0, 0, 1) => {
            let ip_inc = Operand::Immediate(Immediate::Bit8(bytes[1]));
            let instruction = Instruction {
                op: Op::Jno,
                operands: [Some(ip_inc), None],
                length: 2,
            };

            instruction
        }
        // JNS
        (0, 1, 1, 1, 1, 0, 0, 1) => {
            let ip_inc = Operand::Immediate(Immediate::Bit8(bytes[1]));
            let instruction = Instruction {
                op: Op::Jns,
                operands: [Some(ip_inc), None],
                length: 2,
            };

            instruction
        }
        // LOOP
        (1, 1, 1, 0, 0, 0, 1, 0) => {
            let ip_inc = Operand::Immediate(Immediate::Bit8(bytes[1]));
            let instruction = Instruction {
                op: Op::Loop,
                operands: [Some(ip_inc), None],
                length: 2,
            };

            instruction
        }
        // LOOPZ
        (1, 1, 1, 0, 0, 0, 0, 1) => {
            let ip_inc = Operand::Immediate(Immediate::Bit8(bytes[1]));
            let instruction = Instruction {
                op: Op::Loopz,
                operands: [Some(ip_inc), None],
                length: 2,
            };

            instruction
        }
        // LOOPNZ
        (1, 1, 1, 0, 0, 0, 0, 0) => {
            let ip_inc = Operand::Immediate(Immediate::Bit8(bytes[1]));
            let instruction = Instruction {
                op: Op::Loopnz,
                operands: [Some(ip_inc), None],
                length: 2,
            };

            instruction
        }
        // JCXZ
        (1, 1, 1, 0, 0, 0, 1, 1) => {
            let ip_inc = Operand::Immediate(Immediate::Bit8(bytes[1]));
            let instruction = Instruction {
                op: Op::Jcxz,
                operands: [Some(ip_inc), None],
                length: 2,
            };

            instruction
        }
        _ => unimplemented!(),
    }
}

fn decode_register(encoding: u8, wide: u8) -> Operand {
    match wide {
        0 => match encoding {
            0b000 => Operand::Register(Register::AL),
            0b001 => Operand::Register(Register::CL),
            0b010 => Operand::Register(Register::DL),
            0b011 => Operand::Register(Register::BL),
            0b100 => Operand::Register(Register::AH),
            0b101 => Operand::Register(Register::CH),
            0b110 => Operand::Register(Register::DH),
            0b111 => Operand::Register(Register::BH),
            _ => unreachable!(),
        },
        1 => match encoding {
            0b000 => Operand::Register(Register::AX),
            0b001 => Operand::Register(Register::CX),
            0b010 => Operand::Register(Register::DX),
            0b011 => Operand::Register(Register::BX),
            0b100 => Operand::Register(Register::SP),
            0b101 => Operand::Register(Register::BP),
            0b110 => Operand::Register(Register::SI),
            0b111 => Operand::Register(Register::DI),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

fn decode_address(encoding: u8, w: u8, displacement_bytes: &[u8]) -> (Operand, u8) {
    let mut bytes_read = 0;
    let operand_size = MemoryOperandSize::from_w_bit(w);
    let operand_kind = match encoding {
        0b000 => MemoryOperandKind::Direct(MemoryDirect::BX_SI),
        0b001 => MemoryOperandKind::Direct(MemoryDirect::BX_DI),
        0b010 => MemoryOperandKind::Direct(MemoryDirect::BP_SI),
        0b011 => MemoryOperandKind::Direct(MemoryDirect::BP_DI),
        0b100 => MemoryOperandKind::Direct(MemoryDirect::SI),
        0b101 => MemoryOperandKind::Direct(MemoryDirect::DI),
        0b110 => {
            let disp0 = displacement_bytes[0];
            let disp1 = displacement_bytes[1];
            let disp: u16 = ((disp1 as u16) << 8) + disp0 as u16;
            bytes_read = 2;
            MemoryOperandKind::Direct(MemoryDirect::DirectAddress(disp))
        }
        0b111 => MemoryOperandKind::Direct(MemoryDirect::BX),
        _ => unreachable!(),
    };
    let operand = Operand::Memory(MemoryOperand {
        kind: operand_kind,
        size: operand_size,
    });
    (operand, bytes_read)
}

fn decode_address_disp8(encoding: u8, w: u8, disp: i8) -> Operand {
    let operand_size = MemoryOperandSize::from_w_bit(w);
    let operand_kind = match encoding {
        0b000 => MemoryOperandKind::Displacement8bit(MemoryDisplacement8bit::BX_SI(disp)),
        0b001 => MemoryOperandKind::Displacement8bit(MemoryDisplacement8bit::BX_DI(disp)),
        0b010 => MemoryOperandKind::Displacement8bit(MemoryDisplacement8bit::BP_SI(disp)),
        0b011 => MemoryOperandKind::Displacement8bit(MemoryDisplacement8bit::BP_DI(disp)),
        0b100 => MemoryOperandKind::Displacement8bit(MemoryDisplacement8bit::SI(disp)),
        0b101 => MemoryOperandKind::Displacement8bit(MemoryDisplacement8bit::DI(disp)),
        0b110 => MemoryOperandKind::Displacement8bit(MemoryDisplacement8bit::BP(disp)),
        0b111 => MemoryOperandKind::Displacement8bit(MemoryDisplacement8bit::BX(disp)),
        _ => unreachable!(),
    };
    let operand = Operand::Memory(MemoryOperand {
        kind: operand_kind,
        size: operand_size,
    });
    operand
}

fn decode_address_disp16(encoding: u8, w: u8, disp: i16) -> Operand {
    let operand_size = MemoryOperandSize::from_w_bit(w);
    let operand_kind = match encoding {
        0b000 => MemoryOperandKind::Displacement16bit(MemoryDisplacement16bit::BX_SI(disp)),
        0b001 => MemoryOperandKind::Displacement16bit(MemoryDisplacement16bit::BX_DI(disp)),
        0b010 => MemoryOperandKind::Displacement16bit(MemoryDisplacement16bit::BP_SI(disp)),
        0b011 => MemoryOperandKind::Displacement16bit(MemoryDisplacement16bit::BP_DI(disp)),
        0b100 => MemoryOperandKind::Displacement16bit(MemoryDisplacement16bit::SI(disp)),
        0b101 => MemoryOperandKind::Displacement16bit(MemoryDisplacement16bit::DI(disp)),
        0b110 => MemoryOperandKind::Displacement16bit(MemoryDisplacement16bit::BP(disp)),
        0b111 => MemoryOperandKind::Displacement16bit(MemoryDisplacement16bit::BX(disp)),
        _ => unreachable!(),
    };
    let operand = Operand::Memory(MemoryOperand {
        kind: operand_kind,
        size: operand_size,
    });
    operand
}
