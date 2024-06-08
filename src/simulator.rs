use crate::*;

pub fn simulate(registers: &mut RegisterFile, memory: &mut Memory, instruction: Instruction) {
    registers.ip += instruction.length as u16;

    match instruction.op {
        Op::Mov => {
            let dest = instruction.operands[0].expect("mov must have operands");
            let src = instruction.operands[1].expect("mov must have operands");

            match src {
                Operand::Register(reg) => {
                    let (src_value, size, offset) = get_register_value(registers, reg);
                    let src_value = src_value.to_ne_bytes();
                    let end = (size + offset) as usize;
                    let src_slice = &src_value[offset as usize..end];

                    match dest {
                        Operand::Register(reg) => {
                            let dest = get_register_as_slice(registers, reg);
                            dest.copy_from_slice(src_slice);
                        }
                        Operand::Memory(memory_operand) => {
                            let addr = get_address_from_operand(registers, memory_operand);
                            let dest_memory = &mut memory.memory[addr..addr + size as usize];
                            dest_memory.copy_from_slice(src_slice);
                        }
                        Operand::Immediate(_) => unreachable!(),
                    };
                }
                Operand::Immediate(imm) => {
                    let (imm, size) = match imm {
                        Immediate::Bit8(value) => (value as u16, 1),
                        Immediate::Bit16(value) => (value, 2),
                    };
                    let src_slice = u16_as_byte_slice(&imm, size);
                    match dest {
                        Operand::Register(reg) => {
                            let dest = get_register_as_slice(registers, reg);
                            dest.copy_from_slice(src_slice);
                        }
                        Operand::Memory(memory_operand) => {
                            let addr = get_address_from_operand(registers, memory_operand);
                            let dest_memory = &mut memory.memory[addr..addr + size as usize];
                            dest_memory.copy_from_slice(src_slice);
                        }
                        Operand::Immediate(_) => unreachable!(),
                    };
                }
                Operand::Memory(memory_operand) => {
                    let addr = get_address_from_operand(registers, memory_operand);

                    match dest {
                        Operand::Register(reg) => {
                            let dest = get_register_as_slice(registers, reg);
                            let src_memory = &mut memory.memory[addr..addr + dest.len()];
                            dest.copy_from_slice(src_memory);
                        }
                        Operand::Memory(_) => unreachable!(),
                        Operand::Immediate(_) => unreachable!(),
                    };
                }
            };
        }
        Op::Add => {
            let dest = instruction.operands[0].expect("add must have operands");
            let src = instruction.operands[1].expect("add must have operands");

            match src {
                Operand::Register(reg) => {
                    let (src_value, _, offset) = get_register_value(registers, reg);

                    match dest {
                        Operand::Register(reg) => {
                            let (dest_value, size, _) = get_register_value(registers, reg);
                            let sum = src_value + dest_value;

                            let sum_bytes = sum.to_ne_bytes();
                            let end = (size + offset) as usize;
                            let sum_slice = &sum_bytes[offset as usize..end];
                            let dest = get_register_as_slice(registers, reg);
                            dest.copy_from_slice(sum_slice);
                            if size == 1 {
                                set_flags(&mut registers.flags, sum, sum_slice[0], sum_slice[0]);
                            } else {
                                set_flags(&mut registers.flags, sum, sum_slice[0], sum_slice[1]);
                            }
                        }
                        Operand::Memory(_) => unimplemented!(),
                        Operand::Immediate(_) => unreachable!(),
                    };
                }
                Operand::Immediate(imm) => {
                    let (imm, size) = match imm {
                        Immediate::Bit8(value) => (value as u16, 1),
                        Immediate::Bit16(value) => (value, 2),
                    };
                    match dest {
                        Operand::Register(reg) => {
                            let (dest_value, size, offset) = get_register_value(registers, reg);
                            let sum = imm + dest_value;

                            let sum_bytes = sum.to_ne_bytes();
                            let end = (size + offset) as usize;
                            let sum_slice = &sum_bytes[offset as usize..end];
                            let dest = get_register_as_slice(registers, reg);
                            dest.copy_from_slice(sum_slice);
                            if size == 1 {
                                set_flags(&mut registers.flags, sum, sum_slice[0], sum_slice[0]);
                            } else {
                                set_flags(&mut registers.flags, sum, sum_slice[0], sum_slice[1]);
                            }
                        }
                        Operand::Memory(_) => unimplemented!(),
                        Operand::Immediate(_) => unreachable!(),
                    };
                }
                Operand::Memory(_) => unimplemented!(),
            };
        }
        Op::Sub => {
            let dest = instruction.operands[0].expect("sub must have operands");
            let src = instruction.operands[1].expect("sub must have operands");

            match src {
                Operand::Register(reg) => {
                    let (src_value, _, offset) = get_register_value(registers, reg);

                    match dest {
                        Operand::Register(reg) => {
                            let (dest_value, size, _) = get_register_value(registers, reg);
                            let diff = (dest_value as i16 - src_value as i16) as u16;

                            let diff_bytes = diff.to_ne_bytes();
                            let end = (size + offset) as usize;
                            let diff_slice = &diff_bytes[offset as usize..end];
                            let dest = get_register_as_slice(registers, reg);
                            dest.copy_from_slice(diff_slice);

                            if size == 1 {
                                set_flags(&mut registers.flags, diff, diff_slice[0], diff_slice[0]);
                            } else {
                                set_flags(&mut registers.flags, diff, diff_slice[0], diff_slice[1]);
                            }
                        }
                        Operand::Memory(_) => unimplemented!(),
                        Operand::Immediate(_) => unreachable!(),
                    };
                }
                Operand::Immediate(imm) => {
                    let (imm, _) = match imm {
                        Immediate::Bit8(value) => (value as u16, 1),
                        Immediate::Bit16(value) => (value, 2),
                    };
                    match dest {
                        Operand::Register(reg) => {
                            let (dest_value, size, offset) = get_register_value(registers, reg);

                            let diff = (dest_value as i16 - imm as i16) as u16;

                            let diff_bytes = diff.to_ne_bytes();
                            let end = (size + offset) as usize;
                            let diff_slice = &diff_bytes[offset as usize..end];
                            let dest = get_register_as_slice(registers, reg);
                            dest.copy_from_slice(diff_slice);
                            if size == 1 {
                                set_flags(&mut registers.flags, diff, diff_slice[0], diff_slice[0]);
                            } else {
                                set_flags(&mut registers.flags, diff, diff_slice[0], diff_slice[1]);
                            }
                        }
                        Operand::Memory(_) => unimplemented!(),
                        Operand::Immediate(_) => unreachable!(),
                    };
                }
                Operand::Memory(_) => unimplemented!(),
            };
        }
        Op::Cmp => {
            let dest = instruction.operands[0].expect("cmp must have operands");
            let src = instruction.operands[1].expect("cmp must have operands");

            match src {
                Operand::Register(reg) => {
                    let (src_value, _, offset) = get_register_value(registers, reg);

                    match dest {
                        Operand::Register(reg) => {
                            let (dest_value, size, _) = get_register_value(registers, reg);
                            let diff = (dest_value as i16 - src_value as i16) as u16;

                            let diff_bytes = diff.to_ne_bytes();
                            let end = (size + offset) as usize;
                            let diff_slice = &diff_bytes[offset as usize..end];
                            if size == 1 {
                                set_flags(&mut registers.flags, diff, diff_slice[0], diff_slice[0]);
                            } else {
                                set_flags(&mut registers.flags, diff, diff_slice[0], diff_slice[1]);
                            }
                        }
                        Operand::Memory(_) => unimplemented!(),
                        Operand::Immediate(_) => unreachable!(),
                    };
                }
                Operand::Immediate(imm) => {
                    let (imm, _) = match imm {
                        Immediate::Bit8(value) => (value as u16, 1),
                        Immediate::Bit16(value) => (value, 2),
                    };
                    match dest {
                        Operand::Register(reg) => {
                            let (dest_value, size, offset) = get_register_value(registers, reg);

                            let diff = (dest_value as i16 - imm as i16) as u16;

                            let diff_bytes = diff.to_ne_bytes();
                            let end = (size + offset) as usize;
                            let diff_slice = &diff_bytes[offset as usize..end];
                            if size == 1 {
                                set_flags(&mut registers.flags, diff, diff_slice[0], diff_slice[0]);
                            } else {
                                set_flags(&mut registers.flags, diff, diff_slice[0], diff_slice[1]);
                            }
                        }
                        Operand::Memory(_) => unimplemented!(),
                        Operand::Immediate(_) => unreachable!(),
                    };
                }
                Operand::Memory(_) => unimplemented!(),
            };
        }
        Op::Jne => {
            let ip_inc = instruction.operands[0].expect("jne must have operand");
            let value = match ip_inc {
                Operand::Immediate(Immediate::Bit8(imm)) => imm,
                _ => unreachable!(),
            };

            if registers.flags & RegisterFile::ZF_MASK != 0b1000 {
                let test = registers.ip as i16 + value as i8 as i16;
                registers.ip = test as u16;
            }
        }
        _ => unimplemented!(),
    }
}

fn u16_as_byte_slice(value: &u16, size: u8) -> &[u8] {
    unsafe { std::slice::from_raw_parts(value as *const u16 as *const _, size as usize) }
}

fn u16_as_byte_slice_mut(value: &mut u16, size: u8) -> &mut [u8] {
    unsafe { std::slice::from_raw_parts_mut(value as *mut u16 as *mut _, size as usize) }
}

fn set_flags(flags: &mut u16, value: u16, value_lo: u8, value_hi: u8) {
    if value == 0 {
        *flags |= RegisterFile::ZF_MASK;
    } else {
        *flags &= !RegisterFile::ZF_MASK;
    }

    if value_hi & 0x80 == 0x80 {
        *flags |= RegisterFile::SF_MASK;
    } else {
        *flags &= !RegisterFile::SF_MASK;
    }
}

fn get_register_as_slice(registers: &mut RegisterFile, reg: Register) -> &mut [u8] {
    match reg {
        Register::AL => {
            let bytes = u16_as_byte_slice_mut(&mut registers.ax, 2);
            &mut bytes[0..1]
        }
        Register::CL => {
            let bytes = u16_as_byte_slice_mut(&mut registers.cx, 2);
            &mut bytes[0..1]
        }
        Register::DL => {
            let bytes = u16_as_byte_slice_mut(&mut registers.dx, 2);
            &mut bytes[0..1]
        }
        Register::BL => {
            let bytes = u16_as_byte_slice_mut(&mut registers.bx, 2);
            &mut bytes[0..1]
        }
        Register::AH => {
            let bytes = u16_as_byte_slice_mut(&mut registers.ax, 2);
            &mut bytes[1..]
        }
        Register::CH => {
            let bytes = u16_as_byte_slice_mut(&mut registers.cx, 2);
            &mut bytes[1..]
        }
        Register::DH => {
            let bytes = u16_as_byte_slice_mut(&mut registers.dx, 2);
            &mut bytes[1..]
        }
        Register::BH => {
            let bytes = u16_as_byte_slice_mut(&mut registers.bx, 2);
            &mut bytes[1..]
        }

        Register::AX => u16_as_byte_slice_mut(&mut registers.ax, 2),
        Register::CX => u16_as_byte_slice_mut(&mut registers.cx, 2),
        Register::DX => u16_as_byte_slice_mut(&mut registers.dx, 2),
        Register::BX => u16_as_byte_slice_mut(&mut registers.bx, 2),
        Register::SP => u16_as_byte_slice_mut(&mut registers.sp, 2),
        Register::BP => u16_as_byte_slice_mut(&mut registers.bp, 2),
        Register::SI => u16_as_byte_slice_mut(&mut registers.si, 2),
        Register::DI => u16_as_byte_slice_mut(&mut registers.di, 2),
    }
}

fn get_register_value(registers: &mut RegisterFile, reg: Register) -> (u16, u8, u8) {
    match reg {
        Register::AL => (registers.ax, 1, 0),
        Register::CL => (registers.cx, 1, 0),
        Register::DL => (registers.dx, 1, 0),
        Register::BL => (registers.bx, 1, 0),
        Register::AH => (registers.ax, 1, 1),
        Register::CH => (registers.cx, 1, 1),
        Register::DH => (registers.dx, 1, 1),
        Register::BH => (registers.bx, 1, 1),

        Register::AX => (registers.ax, 2, 0),
        Register::CX => (registers.cx, 2, 0),
        Register::DX => (registers.dx, 2, 0),
        Register::BX => (registers.bx, 2, 0),
        Register::SP => (registers.sp, 2, 0),
        Register::BP => (registers.bp, 2, 0),
        Register::SI => (registers.si, 2, 0),
        Register::DI => (registers.di, 2, 0),
    }
}

fn get_address_from_operand(register_file: &RegisterFile, memory_operand: MemoryOperand) -> usize {
    let addr = match memory_operand.kind {
        MemoryOperandKind::Direct(MemoryDirect::BX_SI) => register_file.bx + register_file.si,
        MemoryOperandKind::Direct(MemoryDirect::BX_DI) => register_file.bx + register_file.di,
        MemoryOperandKind::Direct(MemoryDirect::BP_SI) => register_file.bp + register_file.si,
        MemoryOperandKind::Direct(MemoryDirect::BP_DI) => register_file.bp + register_file.di,
        MemoryOperandKind::Direct(MemoryDirect::SI) => register_file.si,
        MemoryOperandKind::Direct(MemoryDirect::DI) => register_file.di,
        MemoryOperandKind::Direct(MemoryDirect::DirectAddress(addr)) => addr,
        MemoryOperandKind::Direct(MemoryDirect::BX) => register_file.bx,

        MemoryOperandKind::Displacement8bit(MemoryDisplacement8bit::BX_SI(disp)) => {
            ((register_file.bx + register_file.si) as i16 + disp as i16) as u16
        }
        MemoryOperandKind::Displacement8bit(MemoryDisplacement8bit::BX_DI(disp)) => {
            ((register_file.bx + register_file.di) as i16 + disp as i16) as u16
        }
        MemoryOperandKind::Displacement8bit(MemoryDisplacement8bit::BP_SI(disp)) => {
            ((register_file.bp + register_file.si) as i16 + disp as i16) as u16
        }
        MemoryOperandKind::Displacement8bit(MemoryDisplacement8bit::BP_DI(disp)) => {
            ((register_file.bp + register_file.di) as i16 + disp as i16) as u16
        }
        MemoryOperandKind::Displacement8bit(MemoryDisplacement8bit::SI(disp)) => {
            (register_file.si as i16 + disp as i16) as u16
        }
        MemoryOperandKind::Displacement8bit(MemoryDisplacement8bit::DI(disp)) => {
            (register_file.di as i16 + disp as i16) as u16
        }
        MemoryOperandKind::Displacement8bit(MemoryDisplacement8bit::BP(disp)) => {
            (register_file.bp as i16 + disp as i16) as u16
        }
        MemoryOperandKind::Displacement8bit(MemoryDisplacement8bit::BX(disp)) => {
            (register_file.bx as i16 + disp as i16) as u16
        }

        MemoryOperandKind::Displacement16bit(MemoryDisplacement16bit::BX_SI(disp)) => {
            ((register_file.bx + register_file.si) as i16 + disp) as u16
        }
        MemoryOperandKind::Displacement16bit(MemoryDisplacement16bit::BX_DI(disp)) => {
            ((register_file.bx + register_file.di) as i16 + disp) as u16
        }
        MemoryOperandKind::Displacement16bit(MemoryDisplacement16bit::BP_SI(disp)) => {
            ((register_file.bp + register_file.si) as i16 + disp) as u16
        }
        MemoryOperandKind::Displacement16bit(MemoryDisplacement16bit::BP_DI(disp)) => {
            ((register_file.bp + register_file.di) as i16 + disp) as u16
        }
        MemoryOperandKind::Displacement16bit(MemoryDisplacement16bit::SI(disp)) => {
            (register_file.si as i16 + disp) as u16
        }
        MemoryOperandKind::Displacement16bit(MemoryDisplacement16bit::DI(disp)) => {
            (register_file.di as i16 + disp) as u16
        }
        MemoryOperandKind::Displacement16bit(MemoryDisplacement16bit::BP(disp)) => {
            (register_file.bp as i16 + disp) as u16
        }
        MemoryOperandKind::Displacement16bit(MemoryDisplacement16bit::BX(disp)) => {
            (register_file.bx as i16 + disp) as u16
        }
    } as usize;

    addr
}
