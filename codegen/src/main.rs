use std::{error::Error, fs::File, io::Write};

use elf64::{
    file_header::{FileHeader, FILE_HEADER_SIZE},
    program::{Phdr, PF_R, PF_W, PF_X, PROGRAM_HEADER_SIZE, PT_LOAD},
};

pub mod elf64;
pub mod limine;

fn main() -> Result<(), Box<dyn Error>> {
    let start_vaddr = 0xffffffff_80000000_u64;

    let program_header_offset = FILE_HEADER_SIZE as u64;
    let data_offset = program_header_offset + 2 * PROGRAM_HEADER_SIZE as u64;

    let data_vaddr = start_vaddr + data_offset;
    let mut data: Vec<u8> = Vec::new();

    let req_vaddr = data_vaddr + data.len() as u64;
    // Align to 8 bytes
    while (data_vaddr + data.len() as u64) % 8 != 0 {
        data.push(0);
    }

    data.extend(bytemuck::bytes_of(&limine::COMMON_MAGIC));
    data.extend(bytemuck::bytes_of(&limine::TERMINAL_REQUEST));
    data.extend(0u64.to_le_bytes()); // Revision
    data.extend(0u64.to_le_bytes()); // Callback
    let terminal_response_vaddr = req_vaddr + data.len() as u64;
    data.extend(0u64.to_le_bytes()); // Response

    data.extend(bytemuck::bytes_of(&limine::COMMON_MAGIC));
    data.extend(bytemuck::bytes_of(&limine::BOOTLOADER_INFO_REQUEST));
    data.extend(0u64.to_le_bytes()); // Revision
    let info_response_vaddr = req_vaddr + data.len() as u64;
    data.extend(0u64.to_le_bytes()); // Response

    let code_vaddr = data_vaddr + data.len() as u64 + (1 << 12);
    let code_offset = data_offset + data.len() as u64;
    let mut code: Vec<u8> = Vec::new();

    let halt_rel = code.len() as isize;
    // jmp halt (infinite loop)
    code.extend([0xeb, 0xfe]);

    let entry = code_vaddr + code.len() as u64;
    // mov rsi, [info_response]
    code.extend([0x48, 0x8B, 0x34, 0x25]);
    code.extend(&info_response_vaddr.to_le_bytes()[..4]);

    // test rsi, rsi
    code.extend([0x48, 0x85, 0xF6]);

    // jz halt
    {
        let end = code.len() as isize + 2;
        code.extend([0x74, i8::try_from(halt_rel - end).unwrap() as u8]);
    }

    // mov rsi, [rsi + 8] (name)
    code.extend([0x48, 0x8B, 0x76, 0x08]);

    // String length

    // xor rdx, rdx
    code.extend([0x48, 0x31, 0xd2]);

    // Loop body
    let mut strlen_loop: Vec<u8> = Vec::new();
    // inc rdx
    strlen_loop.extend([0x48, 0xFF, 0xC2]);

    let loop_start = code.len() as isize;
    // cmp qword ptr [rsi + rdx], 0
    code.extend([0x48, 0x83, 0x3C, 0x16, 0x00]);
    // je loop_end
    {
        let target = strlen_loop.len() as isize + 2;
        code.extend([0x74, i8::try_from(target).unwrap() as u8]);
    }
    code.extend(strlen_loop);
    // jmp loop_start
    {
        let end = code.len() as isize + 2;
        code.extend([0xeb, i8::try_from(loop_start - end).unwrap() as u8]);
    }

    // Terminal write
    // mov rax, [terminal_response]
    code.extend([0x48, 0x8B, 0x04, 0x25]);
    code.extend(&terminal_response_vaddr.to_le_bytes()[..4]);

    // test rax, rax
    code.extend([0x48, 0x85, 0xC0]);

    // jz halt
    {
        let end = code.len() as isize + 2;
        code.extend([0x74, i8::try_from(halt_rel - end).unwrap() as u8]);
    }

    // mov rdi, [rax + 8] (terminal_count)
    code.extend([0x48, 0x8B, 0x78, 0x08]);
    // test rdi, rdi
    code.extend([0x48, 0x85, 0xFF]);
    // jz halt
    {
        let end = code.len() as isize + 2;
        code.extend([0x74, i8::try_from(halt_rel - end).unwrap() as u8]);
    }

    // mov rdi, [rax + 16] (terminals)
    code.extend([0x48, 0x8B, 0x78, 0x10]);
    // mov rdi, [rdi] (terminals[0])
    code.extend([0x48, 0x8B, 0x3F]);
    // mov rax, [rax + 24] (write)
    code.extend([0x48, 0x8B, 0x40, 0x18]);
    // call rax
    code.extend([0xFF, 0xD0]);

    // jmp halt
    {
        let end = code.len() + 2;
        code.extend([
            0xeb,
            i8::try_from(halt_rel as isize - end as isize).unwrap() as u8,
        ]);
    }

    let mut file_header = FileHeader::new();
    file_header.e_machine = 0x3e; // x86_64
    file_header.e_entry = entry;
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
            p_filesz: code.len() as u64,
            p_memsz: code.len() as u64,
            p_align: 1 << 12,
        },
    ];

    let mut file = File::create("kernel.elf")?;
    file.write_all(bytemuck::bytes_of(&file_header))?;
    file.write_all(bytemuck::bytes_of(&program_headers))?;
    file.write_all(&data)?;
    file.write_all(&code)?;

    Ok(())
}
