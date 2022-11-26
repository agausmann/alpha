use std::{error::Error, fs::File, io::Write};

use elf64::{
    file_header::{FileHeader, FILE_HEADER_SIZE},
    program::{Phdr, PF_R, PF_W, PF_X, PROGRAM_HEADER_SIZE, PT_LOAD},
};
use x86::{address::*, instruction::*, register::R64::*, Label};

pub mod elf64;
pub mod limine;
pub mod x86;

fn align_up(x: u64, y: u64) -> u64 {
    if x == 0 {
        0
    } else {
        (1 + (x - 1) / y) * y
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let start_vaddr = 0xffffffff_80000000_u64;

    let program_header_offset = FILE_HEADER_SIZE as u64;
    let program_header_end = program_header_offset + 2 * PROGRAM_HEADER_SIZE as u64;
    let data_offset = align_up(program_header_end, 1 << 12);

    let data_vaddr = start_vaddr;
    let mut data: Vec<u8> = Vec::new();

    // Align to 8 bytes
    while (data_vaddr + data.len() as u64) % 8 != 0 {
        data.push(0);
    }

    data.extend(bytemuck::bytes_of(&limine::COMMON_MAGIC));
    data.extend(bytemuck::bytes_of(&limine::TERMINAL_REQUEST));
    data.extend(0u64.to_le_bytes()); // Revision
    let terminal_response_vaddr = data_vaddr + data.len() as u64;
    data.extend(0u64.to_le_bytes()); // Response
    data.extend(0u64.to_le_bytes()); // Callback

    data.extend(bytemuck::bytes_of(&limine::COMMON_MAGIC));
    data.extend(bytemuck::bytes_of(&limine::BOOTLOADER_INFO_REQUEST));
    data.extend(0u64.to_le_bytes()); // Revision
    let info_response_vaddr = data_vaddr + data.len() as u64;
    data.extend(0u64.to_le_bytes()); // Response

    let code_vaddr = data_vaddr + data.len() as u64 + (1 << 12);
    let code_offset = data_offset + data.len() as u64;

    let mut asm = x86::Assembler::new();
    asm.label("halt");
    asm.push(HLT);
    asm.push(JMP(Label("halt")));

    asm.label("entry");

    asm.push(MOV(RSI, info_response_vaddr));
    asm.push(MOV(RSI, Indirect(RSI)));
    asm.push(TEST(RSI, RSI));
    asm.push(JZ(Label("halt")));

    // .name
    asm.push(MOV(RSI, Index(RSI, 8i8)));

    // String length
    asm.push(XOR(RDX, RDX));
    asm.label("strlen_top");
    asm.push(CMP(Index(RSI, RDX), 0u8));
    asm.push(JZ(Label("strlen_bottom")));
    asm.push(INC(RDX));
    asm.push(JMP(Label("strlen_top")));
    asm.label("strlen_bottom");

    // Terminal write
    asm.push(MOV(RAX, terminal_response_vaddr));
    asm.push(MOV(RAX, Indirect(RAX)));
    asm.push(TEST(RAX, RAX));
    asm.push(JZ(Label("halt")));

    // .terminal_count
    asm.push(MOV(RDI, Index(RAX, 8i8)));
    asm.push(TEST(RDI, RDI));
    asm.push(JZ(Label("halt")));
    // .terminals
    asm.push(MOV(RDI, Index(RAX, 16i8)));
    // [0]
    asm.push(MOV(RDI, Indirect(RDI)));

    // .write
    asm.push(MOV(RAX, Index(RAX, 24i8)));
    asm.push(CALL(RAX));
    asm.push(JMP(Label("halt")));

    let code = asm.finish();

    let mut file_header = FileHeader::new();
    file_header.e_machine = 0x3e; // x86_64
    file_header.e_entry = code_vaddr + u64::try_from(code.label("entry")).unwrap();
    file_header.e_phnum = 2;
    file_header.e_phoff = program_header_offset;

    let program_headers = [
        Phdr {
            p_type: PT_LOAD,
            p_flags: PF_R | PF_W,
            p_offset: data_offset,
            p_vaddr: data_vaddr,
            p_paddr: data_vaddr,
            p_filesz: data.len() as u64,
            p_memsz: data.len() as u64,
            p_align: 1 << 12,
        },
        Phdr {
            p_type: PT_LOAD,
            p_flags: PF_R | PF_X,
            p_offset: code_offset,
            p_vaddr: code_vaddr,
            p_paddr: code_vaddr,
            p_filesz: code.bytes().len() as u64,
            p_memsz: code.bytes().len() as u64,
            p_align: 1 << 12,
        },
    ];

    let mut file = File::create("kernel.elf")?;
    file.write_all(bytemuck::bytes_of(&file_header))?;
    file.write_all(bytemuck::bytes_of(&program_headers))?;
    let padding = vec![0; (data_offset - program_header_end) as usize];
    file.write_all(&padding)?;
    file.write_all(&data)?;
    file.write_all(code.bytes())?;

    Ok(())
}
