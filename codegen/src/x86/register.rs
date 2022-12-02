pub trait Register {
    fn in_opcode(&self) -> u8;
    fn in_rm(&self) -> u8;
    fn in_reg(&self) -> u8;
    fn in_base(&self) -> u8;
    fn in_index(&self) -> u8;

    fn rex_b(&self) -> u8;
    fn rex_x(&self) -> u8;
    fn rex_r(&self) -> u8;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum R8 {
    AL,
    CL,
    DL,
    BL,
    AH,
    CH,
    DH,
    BH,
    //TODO - probably move these to a special register enum
    //BPL,
    //SPL,
    //DIL,
    //SIL,
    R8B,
    R9B,
    R10B,
    R11B,
    R12B,
    R13B,
    R14B,
    R15B,
}

impl R8 {
    fn code(&self) -> u8 {
        match self {
            Self::AL => 0x0,
            Self::CL => 0x1,
            Self::DL => 0x2,
            Self::BL => 0x3,
            Self::AH => 0x4,
            Self::CH => 0x5,
            Self::DH => 0x6,
            Self::BH => 0x7,
            Self::R8B => 0x8,
            Self::R9B => 0x9,
            Self::R10B => 0xa,
            Self::R11B => 0xb,
            Self::R12B => 0xc,
            Self::R13B => 0xd,
            Self::R14B => 0xe,
            Self::R15B => 0xf,
        }
    }

    fn code_3bit(&self) -> u8 {
        self.code() & 0b111
    }

    fn upper_bit(&self) -> u8 {
        self.code() >> 3
    }
}

impl Register for R8 {
    fn in_opcode(&self) -> u8 {
        self.code_3bit() << 0
    }

    fn in_rm(&self) -> u8 {
        // FIXME assert not being used as an address (mod != 0b11)
        self.code_3bit() << 0
    }

    fn in_reg(&self) -> u8 {
        self.code_3bit() << 3
    }

    fn in_base(&self) -> u8 {
        unreachable!("8 bit pointer not supported")
    }

    fn in_index(&self) -> u8 {
        unreachable!("8 bit pointer not supported")
    }

    fn rex_b(&self) -> u8 {
        self.upper_bit() << 0
    }

    fn rex_x(&self) -> u8 {
        self.upper_bit() << 1
    }

    fn rex_r(&self) -> u8 {
        self.upper_bit() << 2
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum R16 {
    AX,
    CX,
    DX,
    BX,
    SP,
    BP,
    SI,
    DI,
    R8W,
    R9W,
    R10W,
    R11W,
    R12W,
    R13W,
    R14W,
    R15W,
}

impl R16 {
    fn code(&self) -> u8 {
        match self {
            Self::AX => 0x0,
            Self::CX => 0x1,
            Self::DX => 0x2,
            Self::BX => 0x3,
            Self::SP => 0x4,
            Self::BP => 0x5,
            Self::SI => 0x6,
            Self::DI => 0x7,
            Self::R8W => 0x8,
            Self::R9W => 0x9,
            Self::R10W => 0xa,
            Self::R11W => 0xb,
            Self::R12W => 0xc,
            Self::R13W => 0xd,
            Self::R14W => 0xe,
            Self::R15W => 0xf,
        }
    }

    fn code_3bit(&self) -> u8 {
        self.code() & 0b111
    }

    fn upper_bit(&self) -> u8 {
        self.code() >> 3
    }
}

impl Register for R16 {
    fn in_opcode(&self) -> u8 {
        self.code_3bit() << 0
    }

    fn in_rm(&self) -> u8 {
        // FIXME assert not being used as an address (mod != 0b11)
        self.code_3bit() << 0
    }

    fn in_reg(&self) -> u8 {
        self.code_3bit() << 3
    }

    fn in_base(&self) -> u8 {
        unreachable!("16 bit pointer not supported")
    }

    fn in_index(&self) -> u8 {
        unreachable!("16 bit pointer not supported")
    }

    fn rex_b(&self) -> u8 {
        self.upper_bit() << 0
    }

    fn rex_x(&self) -> u8 {
        self.upper_bit() << 1
    }

    fn rex_r(&self) -> u8 {
        self.upper_bit() << 2
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum R32 {
    EAX,
    ECX,
    EDX,
    EBX,
    ESP,
    EBP,
    ESI,
    EDI,
    R8D,
    R9D,
    R10D,
    R11D,
    R12D,
    R13D,
    R14D,
    R15D,
}

impl R32 {
    fn code(&self) -> u8 {
        match self {
            Self::EAX => 0x0,
            Self::ECX => 0x1,
            Self::EDX => 0x2,
            Self::EBX => 0x3,
            Self::ESP => 0x4,
            Self::EBP => 0x5,
            Self::ESI => 0x6,
            Self::EDI => 0x7,
            Self::R8D => 0x8,
            Self::R9D => 0x9,
            Self::R10D => 0xa,
            Self::R11D => 0xb,
            Self::R12D => 0xc,
            Self::R13D => 0xd,
            Self::R14D => 0xe,
            Self::R15D => 0xf,
        }
    }

    fn code_3bit(&self) -> u8 {
        self.code() & 0b111
    }

    fn upper_bit(&self) -> u8 {
        self.code() >> 3
    }
}

impl Register for R32 {
    fn in_opcode(&self) -> u8 {
        self.code_3bit() << 0
    }

    fn in_rm(&self) -> u8 {
        // FIXME assert not being used as an address (mod != 0b11)
        self.code_3bit() << 0
    }

    fn in_reg(&self) -> u8 {
        self.code_3bit() << 3
    }

    fn in_base(&self) -> u8 {
        unreachable!("32 bit pointer not supported")
    }

    fn in_index(&self) -> u8 {
        unreachable!("32 bit pointer not supported")
    }

    fn rex_b(&self) -> u8 {
        self.upper_bit() << 0
    }

    fn rex_x(&self) -> u8 {
        self.upper_bit() << 1
    }

    fn rex_r(&self) -> u8 {
        self.upper_bit() << 2
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum R64 {
    RAX,
    RBX,
    RCX,
    RDX,
    RDI,
    RSI,
    RBP,
    RSP,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl R64 {
    fn code(&self) -> u8 {
        match self {
            Self::RAX => 0x0,
            Self::RCX => 0x1,
            Self::RDX => 0x2,
            Self::RBX => 0x3,
            Self::RSP => 0x4,
            Self::RBP => 0x5,
            Self::RSI => 0x6,
            Self::RDI => 0x7,
            Self::R8 => 0x8,
            Self::R9 => 0x9,
            Self::R10 => 0xa,
            Self::R11 => 0xb,
            Self::R12 => 0xc,
            Self::R13 => 0xd,
            Self::R14 => 0xe,
            Self::R15 => 0xf,
        }
    }

    fn code_3bit(&self) -> u8 {
        self.code() & 0b111
    }

    fn upper_bit(&self) -> u8 {
        self.code() >> 3
    }
}

impl Register for R64 {
    fn in_opcode(&self) -> u8 {
        self.code_3bit() << 0
    }

    fn in_rm(&self) -> u8 {
        // FIXME assert not ESP/EBP if being used as an address (mod != 0b11)
        self.code_3bit() << 0
    }

    fn in_reg(&self) -> u8 {
        self.code_3bit() << 3
    }

    fn in_base(&self) -> u8 {
        assert!(*self != Self::RBP, "RBP cannot be used as base");
        self.code_3bit() << 0
    }

    fn in_index(&self) -> u8 {
        assert!(*self != Self::RSP, "RSP cannot be used as index");
        self.code_3bit() << 3
    }

    fn rex_b(&self) -> u8 {
        self.upper_bit() << 0
    }

    fn rex_x(&self) -> u8 {
        self.upper_bit() << 1
    }

    fn rex_r(&self) -> u8 {
        self.upper_bit() << 2
    }
}
