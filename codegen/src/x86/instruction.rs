use super::{
    address::{Index, Indirect},
    register::{Register, R16, R32, R64, R8},
};
use crate::link::{Label, Ptr, Reference, ReferenceFormat};

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

    pub fn operand_size_override(mut self) -> Self {
        self.prefixes.push(0x66);
        self
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

    pub fn op_reg<R: Register>(self, reg: R) -> Self {
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

    pub fn reg<R: Register>(self, reg: R) -> Self {
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

    pub fn rm_reg<R: Register>(self, reg: R) -> Self {
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

    pub fn rm_literal(self, reg: R64) -> Self {
        self.mod_(0b11).rm_reg(reg)
    }

    pub fn indirect(self, indirect: Indirect<R64>) -> Self {
        self.mod_(0b00).rm_reg(indirect.0)
    }

    pub fn indexed_indirect(self, index: Index<R64, R64>) -> Self {
        self.mod_(0b00).rm_const(0b100).index(index.0).base(index.1)
    }

    pub fn indexed_displacement(self, index: Index<R64, i8>) -> Self {
        self.mod_(0b01).rm_reg(index.0).displacement(index.1)
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

    pub fn rip_relative(self, ptr: Ptr<'a>) -> Self {
        self.mod_(0b00)
            .rm_const(0b101)
            .immediate(0u32)
            .reference(Label(ptr.0), ReferenceFormat::Rel32)
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
            .rm_literal(self.0)
    }
}

pub struct RET;

impl<'a> Instruction<'a> for RET {
    fn encode(&self) -> InstructionBuilder<'a> {
        // C3 | RET
        InstructionBuilder::new().opcode(0xc3)
    }
}

pub struct IRET;

impl<'a> Instruction<'a> for IRET {
    fn encode(&self) -> InstructionBuilder<'a> {
        // REX.W + CF | IRETQ
        InstructionBuilder::new().rex_w().opcode(0xcf)
    }
}

pub struct LIDT<Src>(pub Src);

impl<'a> Instruction<'a> for LIDT<Indirect<R64>> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // 0F 01 /3 | LIDT m16&64
        InstructionBuilder::new()
            .opcode([0x0f, 0x01])
            .reg_const(3)
            .indirect(self.0)
    }
}

impl<'a> Instruction<'a> for LIDT<Ptr<'a>> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // 0F 01 /3 | LIDT m16&64
        InstructionBuilder::new()
            .opcode([0x0f, 0x01])
            .reg_const(3)
            .rip_relative(self.0)
    }
}

pub struct STI;

impl<'a> Instruction<'a> for STI {
    fn encode(&self) -> InstructionBuilder<'a> {
        // FB | STI
        InstructionBuilder::new().opcode(0xfb)
    }
}

pub struct NOP;

impl<'a> Instruction<'a> for NOP {
    fn encode(&self) -> InstructionBuilder<'a> {
        // NP 90 | NOP
        InstructionBuilder::new().opcode(0x90)
    }
}

pub struct INT3;

impl<'a> Instruction<'a> for INT3 {
    fn encode(&self) -> InstructionBuilder<'a> {
        // CC | INT3
        InstructionBuilder::new().opcode(0xcc)
    }
}

pub struct PUSH<Src>(pub Src);

impl<'a> Instruction<'a> for PUSH<R64> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // 50+rd | PUSH r64
        InstructionBuilder::new().opcode(0x50).op_reg(self.0)
    }
}

pub struct POP<Dst>(pub Dst);

impl<'a> Instruction<'a> for POP<R64> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // 58+ rd | POP r64
        InstructionBuilder::new().opcode(0x58).op_reg(self.0)
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

impl<'a> Instruction<'a> for MOV<R64, Ptr<'a>> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // REX.W + 8B /r | MOV r64,r/m64
        InstructionBuilder::new()
            .rex_w()
            .opcode(0x8b)
            .reg(self.0)
            .rip_relative(self.1)
    }
}

impl<'a> Instruction<'a> for MOV<R64, R64> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // REX.W + 8B /r | MOV r64,r/m64
        InstructionBuilder::new()
            .rex_w()
            .opcode(0x8b)
            .reg(self.0)
            .rm_literal(self.1)
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
            .indexed_displacement(self.1)
    }
}

impl<'a> Instruction<'a> for MOV<R64, Index<R64, R64>> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // REX.W + 8B /r | MOV r64,r/m64
        InstructionBuilder::new()
            .rex_w()
            .opcode(0x8b)
            .reg(self.0)
            .indexed_indirect(self.1)
    }
}

impl<'a> Instruction<'a> for MOV<R8, Index<R64, R64>> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // 8A /r | MOV r8,r/m8
        // FIXME In 64-bit mode, r/m8 can not be encoded to access the
        // following byte registers if a REX prefix is used: AH, BH, CH, DH.
        InstructionBuilder::new()
            .opcode(0x8a)
            .reg(self.0)
            .indexed_indirect(self.1)
    }
}

impl<'a> Instruction<'a> for MOV<Indirect<R64>, R64> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // REX.W + 89 /r | MOV r/m64,r64
        InstructionBuilder::new()
            .rex_w()
            .opcode(0x89)
            .indirect(self.0)
            .reg(self.1)
    }
}

impl<'a> Instruction<'a> for MOV<Indirect<R64>, R8> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // 88 /r | MOV r/m8,r8
        // FIXME In 64-bit mode, r/m8 can not be encoded to access the
        // following byte registers if a REX prefix is used: AH, BH, CH, DH.
        InstructionBuilder::new()
            .opcode(0x88)
            .indirect(self.0)
            .reg(self.1)
    }
}

impl<'a> Instruction<'a> for MOV<Indirect<R64>, u8> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // C6 /0 ib | MOV r/m8, imm8
        // FIXME In 64-bit mode, r/m8 can not be encoded to access the
        // following byte registers if a REX prefix is used: AH, BH, CH, DH.
        InstructionBuilder::new()
            .rex_w()
            .opcode(0xc6)
            .reg_const(0)
            .indirect(self.0)
            .immediate(self.1)
    }
}

impl<'a> Instruction<'a> for MOV<Index<R64, i8>, R16> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // 89 /r | MOV r/m16,r16
        InstructionBuilder::new()
            .operand_size_override()
            .opcode(0x89)
            .reg(self.1)
            .indexed_displacement(self.0)
    }
}

impl<'a> Instruction<'a> for MOV<Index<R64, i8>, R32> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // 89 /r | MOV r/m32,r32
        InstructionBuilder::new()
            .opcode(0x89)
            .reg(self.1)
            .indexed_displacement(self.0)
    }
}

pub struct LEA<Dst, Src>(pub Dst, pub Src);

impl<'a> Instruction<'a> for LEA<R64, Ptr<'a>> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // REX.W + 8D /r | LEA r64, m
        InstructionBuilder::new()
            .rex_w()
            .opcode(0x8d)
            .reg(self.0)
            .rip_relative(self.1)
    }
}

pub struct SUB<Dst, Src>(pub Dst, pub Src);

impl<'a> Instruction<'a> for SUB<R64, i8> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // 80 /5 ib | SUB r/m8, imm8
        // FIXME In 64-bit mode, r/m8 can not be encoded to access the
        // following byte registers if a REX prefix is used: AH, BH, CH, DH.
        InstructionBuilder::new()
            .opcode(0x80)
            .reg_const(5)
            .rm_literal(self.0)
            .immediate(self.1)
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

pub struct TEST<A, B>(pub A, pub B);

impl<'a> Instruction<'a> for TEST<R64, R64> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // REX.W + 85 /r | TEST r/m64, r64
        InstructionBuilder::new()
            .rex_w()
            .opcode(0x85)
            .rm_literal(self.0)
            .reg(self.1)
    }
}

pub struct OR<Dst, Src>(pub Dst, pub Src);

impl<'a> Instruction<'a> for OR<Index<R64, i8>, i16> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // 81 /1 iw | OR r/m16, imm16
        InstructionBuilder::new()
            .operand_size_override()
            .opcode(0x81)
            .reg_const(1)
            .indexed_displacement(self.0)
            .immediate(self.1)
    }
}

pub struct AND<Dst, Src>(pub Dst, pub Src);

impl<'a> Instruction<'a> for AND<R64, i8> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // REX.W + 83 /4 ib | AND r/m64, imm8
        InstructionBuilder::new()
            .rex_w()
            .opcode(0x83)
            .reg_const(4)
            .rm_literal(self.0)
            .immediate(self.1)
    }
}

pub struct XOR<Dst, Src>(pub Dst, pub Src);

impl<'a> Instruction<'a> for XOR<R64, R64> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // REX.W + 33 /r | XOR r64, r/m64
        InstructionBuilder::new()
            .rex_w()
            .opcode(0x33)
            .reg(self.0)
            .rm_literal(self.1)
    }
}

pub struct SHR<Dst, Amt>(pub Dst, pub Amt);

impl<'a> Instruction<'a> for SHR<R64, i8> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // REX.W + C1 /5 ib | SHR r/m64, imm8
        InstructionBuilder::new()
            .rex_w()
            .opcode(0xc1)
            .reg_const(5)
            .rm_literal(self.0)
            .immediate(self.1)
    }
}

impl<'a> Instruction<'a> for SHR<R64, R8> {
    fn encode(&self) -> InstructionBuilder<'a> {
        // REX.W + D3 /5 | SHR r/m64, CL
        assert!(self.1 == R8::CL, "shift amount must be in CL register");
        InstructionBuilder::new()
            .rex_w()
            .opcode(0xd3)
            .reg_const(5)
            .rm_literal(self.0)
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
            .rm_literal(self.0)
    }
}
