pub mod common {
    pub type Addr = u64;
    pub type Off = u64;
    pub type Half = u16;
    pub type Word = u32;
    pub type Sword = i32;
    pub type Xword = u64;
    pub type Sxword = i64;
    pub type Uchar = u8;
}

pub mod file_header {
    use bytemuck::{Pod, Zeroable};

    use super::{common::*, program::PROGRAM_HEADER_SIZE, section_header::SECTION_HEADER_SIZE};

    pub const MAGIC: &[Uchar; 4] = b"\x7fELF";
    pub const FILE_HEADER_SIZE: Half = 0x40;

    pub const EI_MAG0: usize = 0;
    pub const EI_MAG1: usize = 1;
    pub const EI_MAG2: usize = 2;
    pub const EI_MAG3: usize = 3;
    pub const EI_CLASS: usize = 4;
    pub const EI_DATA: usize = 5;
    pub const EI_VERSION: usize = 6;
    pub const EI_OSABI: usize = 7;
    pub const EI_ABIVERSION: usize = 8;
    pub const EI_PAD: usize = 9;
    pub const EI_NIDENT: usize = 16;

    pub const ELFCLASSNONE: Uchar = 0;
    /// 32-bit objects.
    pub const ELFCLASS32: Uchar = 1;
    /// 64-bit objects.
    pub const ELFCLASS64: Uchar = 2;

    pub const ELFDATANONE: Uchar = 0;
    /// 2's complement, least-significant byte first.
    pub const ELFDATA2LSB: Uchar = 1;
    /// 2's complement, most-significant byte first.
    pub const ELFDATA2MSB: Uchar = 2;

    pub const ELFOSABI_STANDALONE: Uchar = 255;

    /// No file type.
    pub const ET_NONE: Half = 0;
    /// Relocatable object file.
    pub const ET_REL: Half = 1;
    /// Executable file.
    pub const ET_EXEC: Half = 2;
    /// Shared object file.
    pub const ET_DYN: Half = 3;
    /// Core file.
    pub const ET_CORE: Half = 4;
    pub const ET_LOOS: Half = 0xfe00;
    pub const ET_HIOS: Half = 0xfeff;
    pub const ET_LOPROC: Half = 0xff00;
    pub const ET_HIPROC: Half = 0xffff;

    pub const EM_NONE: Half = 0;

    pub const EV_NONE: Uchar = 0;
    pub const EV_CURRENT: Uchar = 1;

    #[derive(Clone, Copy, Pod, Zeroable)]
    #[repr(C)]
    pub struct FileHeader {
        /// ELF identification bytes.
        pub e_ident: [Uchar; EI_NIDENT],

        /// Object file type.
        pub e_type: Half,

        /// Machine type.
        pub e_machine: Half,

        /// Object file version.
        pub e_version: Word,

        /// Entry point address.
        pub e_entry: Addr,

        /// Program header offset.
        pub e_phoff: Off,

        /// Section header offset.
        pub e_shoff: Off,

        /// Processor-specific flags.
        pub e_flags: Word,

        /// ELF header size.
        pub e_ehsize: Half,

        /// Size of program header entry.
        pub e_phentsize: Half,

        /// Number of program header entries.
        pub e_phnum: Half,

        /// Size of section header entry.
        pub e_shentsize: Half,

        /// Number of section header entries.
        pub e_shnum: Half,

        /// Section name string table index.
        pub e_shstrndx: Half,
    }

    impl FileHeader {
        pub fn new() -> Self {
            let mut e_ident = [0; EI_NIDENT];
            e_ident[EI_MAG0..=EI_MAG3].copy_from_slice(MAGIC);
            e_ident[EI_CLASS] = ELFCLASS64;
            e_ident[EI_DATA] = ELFDATA2LSB;
            e_ident[EI_VERSION] = EV_CURRENT;
            e_ident[EI_OSABI] = ELFOSABI_STANDALONE;
            e_ident[EI_ABIVERSION] = 0;

            Self {
                e_ident,
                e_type: ET_NONE,
                e_machine: EM_NONE,
                e_version: EV_CURRENT.into(),
                e_entry: 0,
                e_phoff: 0,
                e_shoff: 0,
                e_flags: 0,
                e_ehsize: FILE_HEADER_SIZE,
                e_phentsize: PROGRAM_HEADER_SIZE,
                e_phnum: 0,
                e_shentsize: SECTION_HEADER_SIZE,
                e_shnum: 0,
                e_shstrndx: 0,
            }
        }
    }
}

pub mod section_header {
    use bytemuck::{Pod, Zeroable};

    use super::common::*;

    /// An undefined or meaningless section reference.
    pub const SHN_UNDEF: Word = 0;
    pub const SHN_LOPROC: Word = 0xff00;
    pub const SHN_HIPROC: Word = 0xff1f;
    pub const SHN_LOOS: Word = 0xff20;
    pub const SHN_HIOS: Word = 0xff3f;

    /// Unused section header.
    pub const SHT_NULL: Word = 0;
    /// Information defined by the program.
    pub const SHT_PROGBITS: Word = 1;
    /// A linker symbol table.
    pub const SHT_SYMTAB: Word = 2;
    pub const SHT_STRTAB: Word = 3;
    pub const SHT_RELA: Word = 4;
    pub const SHT_HASH: Word = 5;
    pub const SHT_DYNAMIC: Word = 6;
    pub const SHT_NOTE: Word = 7;
    pub const SHT_NOBITS: Word = 8;
    pub const SHT_REL: Word = 9;
    pub const SHT_SHLIB: Word = 10;
    pub const SHT_DYNSYM: Word = 11;
    pub const SHT_LOOS: Word = 0x6000_0000;
    pub const SHT_HIOS: Word = 0x6fff_ffff;
    pub const SHT_LOPROC: Word = 0x7000_0000;
    pub const SHT_HIPROC: Word = 0x7fff_ffff;

    pub const SHF_WRITE: Xword = 0x1;
    pub const SHF_ALLOC: Xword = 0x2;
    pub const SHF_EXECINSTR: Xword = 0x4;
    pub const SHF_MASKOS: Xword = 0x0f00_0000;
    pub const SHF_MASKPROC: Xword = 0xf000_0000;

    pub const SECTION_HEADER_SIZE: Half = 0x40;

    #[derive(Clone, Copy, Pod, Zeroable)]
    #[repr(C)]
    pub struct SectionHeader {
        /// The offset, in bytes, to the section name, relative to the start of the
        /// section name string table.
        pub sh_name: Word,

        /// Section type.
        pub sh_type: Word,

        /// Section attributes.
        pub sh_flags: Xword,

        /// The virtual address of the beginning of the section in memory.
        ///
        /// If the section is not allocated to the memory image of the program,
        /// this field should be zero.
        pub sh_addr: Addr,

        /// The offset, in bytes, of the beginning of the section contents in the
        /// file.
        pub sh_offset: Off,

        /// The size in bytes, of the section.
        ///
        /// Except for `SHT_NOBITS` sections, this is the amount of space occupied
        /// in the file.
        pub sh_size: Xword,

        /// The section index of an associated section.
        ///
        /// The meaning of this field depends on the section type:
        ///
        /// - `SHT_DYNAMIC` - String table used by entries in this section.
        /// - `SHT_HASH` - Symbol table to which the hash table applies.
        /// - `SHT_REL` and `SHT_RELA` - Symbol table referenced by relocations.
        /// - `SHT_SYMTAB` and `SHT_DYNSYM` - String table used by entries in
        ///   this section.
        ///
        /// All other section types should set this field to `SHN_UNDEF`.
        pub sh_link: Word,

        /// Extra information about the section.
        ///
        /// The meaning of this field depends on the section type:
        ///
        /// - `SHT_REL` and `SHT_RELA` - Section index of section to which the
        ///   relocations apply.
        /// - `SHT_SYMTAB` and `SHT_DYNSYM` - Index of first non-local symbol
        ///   (i.e., number of local symbols).
        ///
        /// All other section types should set this field to zero `0`.
        pub sh_info: Word,

        /// Required alignment of a section. This field must be a power of two.
        pub sh_addralign: Xword,

        /// The size, in bytes, of each entry, for sections that contain fixed-size
        /// entries. Otherwise, this field contains zero.
        pub sh_entsize: Xword,
    }

    pub struct StandardSection {
        pub name: &'static [u8],
        pub sh_type: Word,
        pub sh_flags: Xword,
    }

    /// Standard section for uninitialized data.
    pub const BSS: StandardSection = StandardSection {
        name: b".bss",
        sh_type: SHT_NOBITS,
        sh_flags: SHF_ALLOC | SHF_WRITE,
    };
    /// Standard section for initialized data.
    pub const DATA: StandardSection = StandardSection {
        name: b".data",
        sh_type: SHT_PROGBITS,
        sh_flags: SHF_ALLOC | SHF_WRITE,
    };
    /// Program interpreter path name.
    pub const INTERP: StandardSection = StandardSection {
        name: b".interp",
        sh_type: SHT_PROGBITS,
        sh_flags: 0,
    };
    /// Read-only data (constants and literals).
    pub const RODATA: StandardSection = StandardSection {
        name: b".rodata",
        sh_type: SHT_PROGBITS,
        sh_flags: SHF_ALLOC,
    };
    /// Executable code.
    pub const TEXT: StandardSection = StandardSection {
        name: b".text",
        sh_type: SHT_PROGBITS,
        sh_flags: SHF_ALLOC | SHF_EXECINSTR,
    };
}

pub mod string_table {
    use super::common::*;

    /// Convenience builder for creating a string table section and calculating
    /// string offsets.
    pub struct StringTableBuilder {
        data: Vec<u8>,
    }

    impl StringTableBuilder {
        /// Create an empty string table builder.
        pub fn new() -> Self {
            Self {
                // The first byte in a string table is defined to be null,
                // so that index 0 always refers to a null or non-existent value.
                data: vec![0],
            }
        }

        /// Appends the given string to the table, and automatically inserts a
        /// null-terminator after it.
        ///
        /// Note: This method does not do any string deduplication.
        ///
        /// Returns the offset of the string relative to the start of the string
        /// table, which can be used to reference the string.
        pub fn push(&mut self, string: &[u8]) -> Word {
            let offset = self.data.len();
            self.data.extend(string);
            self.data.push(0);
            offset.try_into().unwrap()
        }

        /// Finish building the table, and return the raw table data.
        pub fn finish(self) -> Vec<u8> {
            self.data
        }
    }
}

pub mod symbol {
    use bytemuck::{Pod, Zeroable};

    use super::common::*;

    /// Not visible outside object file.
    pub const STB_LOCAL: Uchar = 0 << 4;
    /// Visible to all object files.
    pub const STB_GLOBAL: Uchar = 1 << 4;
    /// Global but with lower precedence than global symbols.
    pub const STB_WEAK: Uchar = 2 << 4;
    pub const STB_LOOS: Uchar = 10 << 4;
    pub const STB_HIOS: Uchar = 12 << 4;
    pub const STB_LOPROC: Uchar = 13 << 4;
    pub const STB_HIPROC: Uchar = 15 << 4;

    /// No type (e.g. absolute symbols)
    pub const STT_NOTYPE: Uchar = 0;
    /// Data object.
    pub const STT_OBJECT: Uchar = 1;
    /// Function entry point.
    pub const STT_FUNC: Uchar = 2;
    /// Section.
    pub const STT_SECTION: Uchar = 3;

    pub const SYMBOL_SIZE: Half = 0x18;

    #[derive(Clone, Copy, Pod, Zeroable)]
    #[repr(C)]
    pub struct Symbol {
        /// The offset, in bytes, to the symbol name, relative to the start of
        /// the symbol string table.
        pub st_name: Word,

        /// Type and Binding attributes.
        pub st_info: Uchar,

        /// Reserved. Set to zero.
        pub st_other: Uchar,

        /// Section index of the section in which the symbol is "defined".
        ///
        /// For undefined symbols, this field contains `SHN_UNDEF`;
        /// for absolute symbols, it contains SHN_ABS;
        /// for common symbols, it contains SHN_COMMON.
        pub st_shndx: Half,

        /// Symbol value.
        ///
        /// This may be an absolute value or a relocatable address.
        ///
        /// In relocatable files, this field contains the alignment constraint
        /// for common symbols, and a section-relative offset for defined
        /// relocatable symbols.
        ///
        /// In executable and shared object files, this field contains a
        /// virtual address for defined relocatable symbols.
        pub st_value: Addr,

        /// Size associated with the symbol. If a symbol does not have an
        /// associated size, or the size is unknown, this field contains zero.
        pub st_size: Xword,
    }
}

pub mod reloc {
    use super::common::*;

    use bytemuck::{Pod, Zeroable};

    #[derive(Clone, Copy, Pod, Zeroable)]
    #[repr(C)]
    pub struct Rel {
        /// Address of reference.
        pub r_offset: Addr,
        /// Symbol index and type of relocation.
        pub r_info: Xword,
    }

    impl Rel {
        pub fn r_sym(&self) -> Word {
            (self.r_info >> 32) as Word
        }

        pub fn r_type(&self) -> Word {
            (self.r_info >> 0) as Word
        }
    }

    #[derive(Clone, Copy, Pod, Zeroable)]
    #[repr(C)]
    pub struct Rela {
        /// Address of reference.
        pub r_offset: Addr,
        /// Symbol index and type of relocation.
        pub r_info: Xword,
        /// Constant part of expression.
        pub r_addend: Sxword,
    }

    impl Rela {
        pub fn r_sym(&self) -> Word {
            (self.r_info >> 32) as Word
        }

        pub fn r_type(&self) -> Word {
            (self.r_info >> 0) as Word
        }
    }

    pub fn r_info(r_sym: Word, r_type: Word) -> Xword {
        ((r_sym as Xword) << 32) | ((r_type as Xword) << 0)
    }
}

pub mod program {
    use super::common::*;

    use bytemuck::{Pod, Zeroable};

    pub const PT_NULL: Word = 0;
    pub const PT_LOAD: Word = 1;
    pub const PT_DYNAMIC: Word = 2;
    pub const PT_INTERP: Word = 3;
    pub const PT_NOTE: Word = 4;
    pub const PT_SHLIB: Word = 5;
    pub const PT_PHDR: Word = 6;
    pub const PT_LOOS: Word = 0x6000_0000;
    pub const PT_HIOS: Word = 0x6fff_ffff;
    pub const PT_LOPROC: Word = 0x7000_0000;
    pub const PT_HIPROC: Word = 0x7fff_ffff;

    pub const PF_X: Word = 0x1;
    pub const PF_W: Word = 0x2;
    pub const PF_R: Word = 0x4;
    pub const PF_MASKOS: Word = 0x00ff_0000;
    pub const PF_MASKPROC: Word = 0xff00_0000;

    pub const PROGRAM_HEADER_SIZE: Half = 0x38;

    #[derive(Clone, Copy, Pod, Zeroable)]
    #[repr(C)]
    pub struct Phdr {
        /// Type of segment.
        pub p_type: Word,
        /// Segment attributes.
        pub p_flags: Word,
        /// Offset in file.
        pub p_offset: Off,
        /// Virtual address in memory.
        pub p_vaddr: Addr,
        /// Reserved for systems with physical addressing.
        pub p_paddr: Addr,
        /// Size of segment in file.
        pub p_filesz: Xword,
        /// Size of segment in memory.
        pub p_memsz: Xword,
        /// Alignment of segment. Must be a power of two.
        ///
        /// The values of `p_offset` and `p_vaddr` must be congruent modulo
        /// `p_align`; i.e. `p_offset % p_align == p_vaddr % p_align`
        pub p_align: Xword,
    }
}

#[cfg(test)]
mod tests {
    use crate::elf64::program::{Phdr, PROGRAM_HEADER_SIZE};

    use super::{
        file_header::{FileHeader, FILE_HEADER_SIZE},
        section_header::{SectionHeader, SECTION_HEADER_SIZE},
        symbol::{Symbol, SYMBOL_SIZE},
    };

    use std::mem::size_of;

    #[test]
    fn file_header_size() {
        // Sum of field sizes should be 64 bytes.
        assert_eq!(
            size_of::<FileHeader>(),
            usize::try_from(FILE_HEADER_SIZE).unwrap()
        );
    }

    #[test]
    fn section_header_size() {
        assert_eq!(
            size_of::<SectionHeader>(),
            usize::try_from(SECTION_HEADER_SIZE).unwrap()
        );
    }

    #[test]
    fn program_header_size() {
        assert_eq!(
            size_of::<Phdr>(),
            usize::try_from(PROGRAM_HEADER_SIZE).unwrap()
        );
    }

    #[test]
    fn symbol_size() {
        assert_eq!(size_of::<Symbol>(), usize::try_from(SYMBOL_SIZE).unwrap());
    }
}
