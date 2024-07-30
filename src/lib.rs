use std::fmt::{self, Display};

mod decoder;
pub use decoder::decode_instruction;

mod simulator;
pub use simulator::simulate;

#[derive(Default, Debug, Clone, Copy)]
pub struct RegisterFile {
    pub ax: u16,
    pub bx: u16,
    pub cx: u16,
    pub dx: u16,

    pub sp: u16,
    pub bp: u16,
    pub si: u16,
    pub di: u16,

    pub ip: u16,
    pub flags: u16,
}

impl RegisterFile {
    const ZF_MASK: u16 = 1 << 3;
    const SF_MASK: u16 = 1 << 4;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Instruction {
    pub op: Op,
    pub length: u8,
    pub operands: [Option<Operand>; 2],
}

impl Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.op {
            Op::Mov => write!(f, "mov")?,
            Op::Add => write!(f, "add")?,
            Op::Sub => write!(f, "sub")?,
            Op::Cmp => write!(f, "cmp")?,
            Op::Je => write!(f, "je")?,
            Op::Jl => write!(f, "jl")?,
            Op::Jle => write!(f, "jle")?,
            Op::Jb => write!(f, "jb")?,
            Op::Jbe => write!(f, "jba")?,
            Op::Jp => write!(f, "jp")?,
            Op::Jo => write!(f, "jo")?,
            Op::Js => write!(f, "js")?,
            Op::Jne => write!(f, "jne")?,
            Op::Jnl => write!(f, "jnl")?,
            Op::Jg => write!(f, "jg")?,
            Op::Jnb => write!(f, "jnb")?,
            Op::Ja => write!(f, "ja")?,
            Op::Jnp => write!(f, "jnp")?,
            Op::Jno => write!(f, "jno")?,
            Op::Jns => write!(f, "jns")?,
            Op::Loop => write!(f, "loop")?,
            Op::Loopz => write!(f, "loopz")?,
            Op::Loopnz => write!(f, "loopnz")?,
            Op::Jcxz => write!(f, "jcxz")?,
        };

        if let Some(operand) = &self.operands[0] {
            write!(f, " {operand}")?;
        }

        if let Some(operand) = &self.operands[1] {
            write!(f, ", {operand}")?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Mov,
    Add,
    Sub,
    Cmp,
    Je,
    Jl,
    Jle,
    Jb,
    Jbe,
    Jp,
    Jo,
    Js,
    Jne,
    Jnl,
    Jg,
    Jnb,
    Ja,
    Jnp,
    Jno,
    Jns,
    Loop,
    Loopz,
    Loopnz,
    Jcxz,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operand {
    Register(Register),
    Memory(MemoryOperand),
    Immediate(Immediate),
}

impl Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Register(reg) => match reg {
                Register::AL => write!(f, "al")?,
                Register::CL => write!(f, "cl")?,
                Register::DL => write!(f, "dl")?,
                Register::BL => write!(f, "bl")?,
                Register::AH => write!(f, "ah")?,
                Register::CH => write!(f, "ch")?,
                Register::DH => write!(f, "dh")?,
                Register::BH => write!(f, "bh")?,

                Register::AX => write!(f, "ax")?,
                Register::CX => write!(f, "cx")?,
                Register::DX => write!(f, "dx")?,
                Register::BX => write!(f, "bx")?,
                Register::SP => write!(f, "sp")?,
                Register::BP => write!(f, "bp")?,
                Register::SI => write!(f, "si")?,
                Register::DI => write!(f, "di")?,
            },
            Operand::Memory(mem) => match mem.kind {
                MemoryOperandKind::Direct_BX_SI => write!(f, "{} [bx + si]", mem.size)?,
                MemoryOperandKind::Direct_BX_DI => write!(f, "{} [bx + di]", mem.size)?,
                MemoryOperandKind::Direct_BP_SI => write!(f, "{} [bp + si]", mem.size)?,
                MemoryOperandKind::Direct_BP_DI => write!(f, "{} [bp + di]", mem.size)?,
                MemoryOperandKind::Direct_SI => write!(f, "{} [si]", mem.size)?,
                MemoryOperandKind::Direct_DI => write!(f, "{} [di]", mem.size)?,
                MemoryOperandKind::Direct_Address(value) => write!(f, "{} [{value}]", mem.size)?,
                MemoryOperandKind::Direct_BX => write!(f, "{} [bx]", mem.size)?,

                MemoryOperandKind::Disp8_BX_SI(disp) => {
                    write!(f, "{} [bx + si {:+}]", mem.size, disp)?
                }
                MemoryOperandKind::Disp8_BX_DI(disp) => {
                    write!(f, "{} [bx + di {:+}]", mem.size, disp)?
                }
                MemoryOperandKind::Disp8_BP_SI(disp) => {
                    write!(f, "{} [bp + si {:+}]", mem.size, disp)?
                }
                MemoryOperandKind::Disp8_BP_DI(disp) => {
                    write!(f, "{} [bp + di {:+}]", mem.size, disp)?
                }
                MemoryOperandKind::Disp8_SI(disp) => write!(f, "{} [si {:+}]", mem.size, disp)?,
                MemoryOperandKind::Disp8_DI(disp) => write!(f, "{} [di {:+}]", mem.size, disp)?,
                MemoryOperandKind::Disp8_BP(disp) => write!(f, "{} [bp {:+}]", mem.size, disp)?,
                MemoryOperandKind::Disp8_BX(disp) => write!(f, "{} [bx {:+}]", mem.size, disp)?,

                MemoryOperandKind::Disp16_BX_SI(disp) => {
                    write!(f, "{} [bx + si {:+}]", mem.size, disp)?
                }
                MemoryOperandKind::Disp16_BX_DI(disp) => {
                    write!(f, "{} [bx + di {:+}]", mem.size, disp)?
                }
                MemoryOperandKind::Disp16_BP_SI(disp) => {
                    write!(f, "{} [bp + si {:+}]", mem.size, disp)?
                }
                MemoryOperandKind::Disp16_BP_DI(disp) => {
                    write!(f, "{} [bp + di {:+}]", mem.size, disp)?
                }
                MemoryOperandKind::Disp16_SI(disp) => write!(f, "{} [si {:+}]", mem.size, disp)?,
                MemoryOperandKind::Disp16_DI(disp) => write!(f, "{} [di {:+}]", mem.size, disp)?,
                MemoryOperandKind::Disp16_BP(disp) => write!(f, "{} [bp {:+}]", mem.size, disp)?,
                MemoryOperandKind::Disp16_BX(disp) => write!(f, "{} [bx {:+}]", mem.size, disp)?,
            },
            Operand::Immediate(imm) => match imm {
                Immediate::Bit8(imm) => write!(f, "byte {}", imm)?,
                Immediate::Bit16(imm) => write!(f, "word {}", imm)?,
            },
        };
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Register {
    AL,
    CL,
    DL,
    BL,
    AH,
    CH,
    DH,
    BH,

    AX,
    CX,
    DX,
    BX,
    SP,
    BP,
    SI,
    DI,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Immediate {
    Bit8(u8),
    Bit16(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryOperand {
    kind: MemoryOperandKind,
    size: MemoryOperandSize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryOperandSize {
    Word,
    Byte,
}

impl MemoryOperandSize {
    fn from_w_bit(w: u8) -> Self {
        match w {
            0 => Self::Byte,
            1 => Self::Word,
            _ => unreachable!(),
        }
    }
}

impl Display for MemoryOperandSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Byte => write!(f, "byte"),
            Self::Word => write!(f, "word"),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryOperandKind {
    Direct_BX_SI,
    Direct_BX_DI,
    Direct_BP_SI,
    Direct_BP_DI,
    Direct_SI,
    Direct_DI,
    Direct_Address(u16),
    Direct_BX,

    Disp8_BX_SI(i8),
    Disp8_BX_DI(i8),
    Disp8_BP_SI(i8),
    Disp8_BP_DI(i8),
    Disp8_SI(i8),
    Disp8_DI(i8),
    Disp8_BP(i8),
    Disp8_BX(i8),

    Disp16_BX_SI(i16),
    Disp16_BX_DI(i16),
    Disp16_BP_SI(i16),
    Disp16_BP_DI(i16),
    Disp16_SI(i16),
    Disp16_DI(i16),
    Disp16_BP(i16),
    Disp16_BX(i16),
    // Direct(MemoryDirect),
    // Displacement8bit(MemoryDisplacement8bit),
    // Displacement16bit(MemoryDisplacement16bit),
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryDirect {
    BX_SI,
    BX_DI,
    BP_SI,
    BP_DI,
    SI,
    DI,
    DirectAddress(u16),
    BX,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryDisplacement8bit {
    BX_SI(i8),
    BX_DI(i8),
    BP_SI(i8),
    BP_DI(i8),
    SI(i8),
    DI(i8),
    BP(i8),
    BX(i8),
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryDisplacement16bit {
    BX_SI(i16),
    BX_DI(i16),
    BP_SI(i16),
    BP_DI(i16),
    SI(i16),
    DI(i16),
    BP(i16),
    BX(i16),
}
