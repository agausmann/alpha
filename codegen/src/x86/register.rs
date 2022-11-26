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
    BPL,
    SPL,
    DIL,
    SIL,
    R8B,
    R9B,
    R10B,
    R11B,
    R12B,
    R13B,
    R14B,
    R15B,
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
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
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
    pub fn code(&self) -> u8 {
        match self {
            Self::RAX | Self::R8 => 0x00,
            Self::RCX | Self::R9 => 0x01,
            Self::RDX | Self::R10 => 0x02,
            Self::RBX | Self::R11 => 0x03,
            Self::RSP | Self::R12 => 0x04,
            Self::RBP | Self::R13 => 0x05,
            Self::RSI | Self::R14 => 0x06,
            Self::RDI | Self::R15 => 0x07,
        }
    }

    pub fn in_opcode(&self) -> u8 {
        self.code() << 0
    }

    pub fn in_rm(&self) -> u8 {
        self.code() << 0
    }

    pub fn in_reg(&self) -> u8 {
        self.code() << 3
    }

    pub fn in_base(&self) -> u8 {
        self.code() << 0
    }

    pub fn in_index(&self) -> u8 {
        self.code() << 3
    }

    pub fn upper_bit(&self) -> u8 {
        match self {
            Self::RAX
            | Self::RCX
            | Self::RDX
            | Self::RBX
            | Self::RSP
            | Self::RBP
            | Self::RSI
            | Self::RDI => 0x00,
            Self::R8
            | Self::R9
            | Self::R10
            | Self::R11
            | Self::R12
            | Self::R13
            | Self::R14
            | Self::R15 => 0x01,
        }
    }

    pub fn rex_b(&self) -> u8 {
        self.upper_bit() << 0
    }

    pub fn rex_x(&self) -> u8 {
        self.upper_bit() << 1
    }

    pub fn rex_r(&self) -> u8 {
        self.upper_bit() << 2
    }
}
