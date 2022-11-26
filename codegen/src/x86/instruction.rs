use super::{
    address::{Index, Indirect},
    register::R64,
    Label, Reference, ReferenceFormat,
};

pub struct InstructionBuilder<'a> {
    prefixes: Vec<u8>,
    rex: u8,
    opcode_size: u8,
    opcode: [u8; 3],
    modrm: Option<u8>,
    sib: Option<u8>,
    displacement: Option<Immediate>,
    immediate: Option<Immediate>,
    reference: Option<(Label<'a>, ReferenceFormat)>,
}

impl<'a> InstructionBuilder<'a> {
    pub fn new() -> Self {
        Self {
            prefixes: Vec::new(),
            rex: 0x40,
            opcode_size: 0,
            opcode: [0; 3],
            modrm: None,
            sib: None,
            displacement: None,
            immediate: None,
            reference: None,
        }
    }

    pub fn rex_w(self) -> Self {
        Self {
            rex: self.rex | 0x08,
            ..self
        }
    }

    pub fn opcode<O: Opcode>(self, opcode: O) -> Self {
        Self {
            opcode_size: opcode.size(),
            opcode: opcode.pad_start(),
            ..self
        }
    }

    pub fn op_reg(self, reg: R64) -> Self {
        Self {
            rex: self.rex | reg.rex_b(),
            opcode: [
                self.opcode[0],
                self.opcode[1],
                self.opcode[2] | reg.in_opcode(),
            ],
            ..self
        }
    }

    pub fn mod_(self, mod_: u8) -> Self {
        Self {
            modrm: Some(self.modrm.unwrap_or(0x00) | mod_ << 6),
            ..self
        }
    }

    pub fn reg(self, reg: R64) -> Self {
        Self {
            rex: self.rex | reg.rex_r(),
            modrm: Some(self.modrm.unwrap_or(0x00) | reg.in_reg()),
            ..self
        }
    }

    pub fn reg_const(self, x: u8) -> Self {
        Self {
            modrm: Some(self.modrm.unwrap_or(0x00) | (x << 3)),
            ..self
        }
    }

    pub fn rm_reg(self, reg: R64) -> Self {
        Self {
            rex: self.rex | reg.rex_b(),
            modrm: Some(self.modrm.unwrap_or(0x00) | reg.in_rm()),
            ..self
        }
    }

    pub fn rm_const(self, x: u8) -> Self {
        Self {
            modrm: Some(self.modrm.unwrap_or(0x00) | (x << 0)),
            ..self
        }
    }

    pub fn index(self, reg: R64) -> Self {
        Self {
            rex: self.rex | reg.rex_x(),
            sib: Some(self.sib.unwrap_or(0x00) | reg.in_reg()),
            ..self
        }
    }

    pub fn base(self, reg: R64) -> Self {
        Self {
            rex: self.rex | reg.rex_b(),
            sib: Some(self.sib.unwrap_or(0x00) | reg.in_base()),
            ..self
        }
    }

    pub fn displacement<I: Into<Immediate>>(self, displacement: I) -> Self {
        Self {
            displacement: Some(displacement.into()),
            ..self
        }
    }

    pub fn immediate<I: Into<Immediate>>(self, immediate: I) -> Self {
        Self {
            immediate: Some(immediate.into()),
            ..self
        }
    }

    pub fn indirect(self, indirect: Indirect<R64>) -> Self {
        self.mod_(0b00).rm_reg(indirect.0)
    }

    pub fn indexed_indirect(self, index: Index<R64, R64>) -> Self {
        self.mod_(0b00).rm_const(0b100).index(index.0).base(index.1)
    }

    pub fn reference(self, label: Label<'a>, format: ReferenceFormat) -> Self {
        Self {
            reference: Some((label, format)),
            ..self
        }
    }

    pub fn rel32(self, label: Label<'a>) -> Self {
        self.displacement(0i32)
            .reference(label, ReferenceFormat::Rel32)
    }

    pub fn serialize<'b>(&'b self) -> impl IntoIterator<Item = u8> + 'b {
        self.prefixes
            .iter()
            .copied()
            .chain(if self.rex & 0x0f != 0 {
                Some(self.rex)
            } else {
                None
            })
            .chain(
                self.opcode[(self.opcode.len() - self.opcode_size as usize)..]
                    .iter()
                    .copied(),
            )
            .chain(self.modrm)
            .chain(self.sib)
            .chain(self.displacement.iter().flat_map(Immediate::bytes).copied())
            .chain(self.immediate.iter().flat_map(Immediate::bytes).copied())
    }

    pub fn references(&self) -> impl IntoIterator<Item = (Label<'a>, Reference)> {
        // FIXME: This assumes that the reference is at the end of the instruction.
        let size = self.serialize().into_iter().count();
        self.reference.into_iter().map(move |(label, format)| {
            (
                label,
                Reference {
                    location: size - format.len(),
                    format,
                },
            )
        })
    }
}

pub enum Immediate {
    X8([u8; 1]),
    X16([u8; 2]),
    X32([u8; 4]),
    X64([u8; 8]),
}

impl Immediate {
    fn bytes(&self) -> &[u8] {
        match self {
            Self::X8(arr) => arr.as_slice(),
            Self::X16(arr) => arr.as_slice(),
            Self::X32(arr) => arr.as_slice(),
            Self::X64(arr) => arr.as_slice(),
        }
    }
}

macro_rules! immediate_conversions {
    ($($t:ty: $x:ident,)*) => {$(
        impl From<$t> for Immediate {
            fn from(val: $t) -> Self {
                Self::$x(val.to_le_bytes())
            }
        }
    )*}
}

immediate_conversions! {
    i8: X8,
    u8: X8,
    i16: X16,
    u16: X16,
    i32: X32,
    u32: X32,
    i64: X64,
    u64: X64,
}

pub trait Opcode {
    fn size(&self) -> u8;
    fn pad_start(&self) -> [u8; 3];
}

impl Opcode for u8 {
    fn size(&self) -> u8 {
        1
    }

    fn pad_start(&self) -> [u8; 3] {
        [0, 0, *self]
    }
}

impl Opcode for [u8; 2] {
    fn size(&self) -> u8 {
        2
    }

    fn pad_start(&self) -> [u8; 3] {
        [0, self[0], self[1]]
    }
}

impl Opcode for [u8; 3] {
    fn size(&self) -> u8 {
        3
    }

    fn pad_start(&self) -> [u8; 3] {
        *self
    }
}

pub trait Instruction<'a> {
    fn encode(&self) -> InstructionBuilder<'a>;
}

pub struct HLT;

impl<'a> Instruction<'a> for HLT {
    fn encode(&self) -> InstructionBuilder<'a> {
        // F4 | HLT
        InstructionBuilder::new().opcode(0xf4)
    }
}

pub struct JMP<Target>(pub Target);

impl<'a> Instruction<'a> for JMP<Label<'a>> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // E9 cd | JMP rel32
        InstructionBuilder::new().opcode(0xe9).rel32(self.0)
    }
}

pub struct JZ<Target>(pub Target);

impl<'a> Instruction<'a> for JZ<Label<'a>> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // 0F 84 cd | JZ rel32
        InstructionBuilder::new().opcode([0x0f, 0x84]).rel32(self.0)
    }
}

pub struct CALL<Target>(pub Target);

impl<'a> Instruction<'a> for CALL<Label<'a>> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // E8 cd | CALL rel32
        InstructionBuilder::new().opcode(0xe8).rel32(self.0)
    }
}

impl<'a> Instruction<'a> for CALL<R64> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // FF /2 | CALL r/m64
        InstructionBuilder::new()
            .opcode(0xff)
            .reg_const(2)
            .mod_(0b11)
            .rm_reg(self.0)
    }
}

pub struct RET;

impl<'a> Instruction<'a> for RET {
    fn encode(&self) -> InstructionBuilder<'a> {
        // C3 | RET
        InstructionBuilder::new().opcode(0xc3)
    }
}

pub struct CMP<A, B>(pub A, pub B);

impl<'a> Instruction<'a> for CMP<Index<R64, R64>, u8> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // 80 /7 ib | CMP r/m8, imm8
        InstructionBuilder::new()
            .opcode(0x80)
            .reg_const(7)
            .indexed_indirect(self.0)
            .immediate(self.1)
    }
}

pub struct MOV<Dst, Src>(pub Dst, pub Src);

impl<'a> Instruction<'a> for MOV<R64, u64> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // REX.W + B8+ rd io | MOV r64, imm64
        InstructionBuilder::new()
            .rex_w()
            .opcode(0xb8)
            .op_reg(self.0)
            .immediate(self.1)
    }
}

impl<'a> Instruction<'a> for MOV<R64, Indirect<R64>> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // REX.W + 8B /r | MOV r64,r/m64
        InstructionBuilder::new()
            .rex_w()
            .opcode(0x8b)
            .reg(self.0)
            .indirect(self.1)
    }
}

impl<'a> Instruction<'a> for MOV<R64, Index<R64, i8>> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // REX.W + 8B /r | MOV r64,r/m64
        InstructionBuilder::new()
            .rex_w()
            .opcode(0x8b)
            .reg(self.0)
            .mod_(0b01)
            .rm_reg(self.1 .0)
            .displacement(self.1 .1)
    }
}

pub struct TEST<A, B>(pub A, pub B);

impl<'a> Instruction<'a> for TEST<R64, R64> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // REX.W + 85 /r | TEST r/m64, r64
        InstructionBuilder::new()
            .rex_w()
            .opcode(0x85)
            .mod_(0b11)
            .rm_reg(self.0)
            .reg(self.1)
    }
}

pub struct XOR<Dst, Src>(pub Dst, pub Src);

impl<'a> Instruction<'a> for XOR<R64, R64> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // REX.W + 33 /r | XOR r64, r/m64
        InstructionBuilder::new()
            .rex_w()
            .opcode(0x33)
            .mod_(0b11)
            .reg(self.0)
            .rm_reg(self.1)
    }
}

pub struct INC<Dst>(pub Dst);

impl<'a> Instruction<'a> for INC<R64> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // REX.W + FF /0 | INC r/m64
        InstructionBuilder::new()
            .rex_w()
            .opcode(0xff)
            .reg_const(0)
            .mod_(0b11)
            .rm_reg(self.0)
    }
}
