use std::{error::Error, fs::File};

use elf64::program::{PF_R, PF_W, PF_X};
use link::{ElfLinker, Label, Ptr, ReferenceFormat, Segment};
use x86::{
    address::*,
    instruction::*,
    register::{R64::*, R8::*},
};

pub mod elf64;
pub mod limine;
pub mod link;
pub mod math;
pub mod x86;

fn main() -> Result<(), Box<dyn Error>> {
    let mut data = Segment::new();

    data.align(8);

    data.offset_label(limine::RESPONSE_OFFSET, "terminal_response");
    data.append(&limine::Request::new(limine::TERMINAL_REQUEST, 0));
    data.append_reference("terminal_callback", ReferenceFormat::Abs64);

    data.offset_label(limine::RESPONSE_OFFSET, "bootloader_info_response");
    data.append(&limine::Request::new(limine::BOOTLOADER_INFO_REQUEST, 0));

    data.label("str_hello");
    data.append(b"Hello \0");

    data.label("str_space");
    data.append(b" \0");

    data.label("tohex_lut");
    data.append(b"0123456789abcdef");

    // TODO move to bss segment
    data.label("tohex_buffer");
    data.append(&[0u8; 32]);

    let mut asm = x86::Assembler::new();

    // Entrypoint
    asm.label("entry");

    asm.push(MOV(RBX, Ptr("bootloader_info_response")));
    asm.push(MOV(RBX, Indirect(RBX)));
    asm.push(TEST(RBX, RBX));
    asm.push(JZ(Label("halt")));

    asm.push(MOV(RSI, Ptr("str_hello")));
    asm.push(CALL(Label("print")));

    // .name
    asm.push(MOV(RSI, Index(RBX, 8i8)));
    asm.push(CALL(Label("print")));

    asm.push(MOV(RSI, Ptr("str_space")));
    asm.push(CALL(Label("print")));

    // .version
    asm.push(MOV(RSI, Index(RBX, 16i8)));
    asm.push(CALL(Label("print")));

    asm.push(MOV(RSI, Ptr("str_space")));
    asm.push(CALL(Label("print")));

    asm.push(MOV(RDI, 0xdeadbeef_u64));
    asm.push(CALL(Label("tohex")));
    asm.push(MOV(RSI, RAX));
    asm.push(CALL(Label("print")));

    asm.push(JMP(Label("halt")));

    // Print procedure
    // - RSI - String to print
    asm.label("print");

    // String length
    asm.push(XOR(RDX, RDX));
    asm.label("strlen_top");
    asm.push(CMP(Index(RSI, RDX), 0u8));
    asm.push(JZ(Label("strlen_bottom")));
    asm.push(INC(RDX));
    asm.push(JMP(Label("strlen_top")));
    asm.label("strlen_bottom");

    // Terminal write
    asm.push(MOV(RAX, Ptr("terminal_response")));
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

    asm.push(RET);

    // Integer to hex string
    // - RDI - 64-bit integer value to format
    // - Output - RAX - Pointer to null-terminated string
    // Pointer only contains valid data until next call
    asm.label("tohex");
    // TODO relax RCX to a smaller register size
    asm.push(MOV(RCX, 64));
    asm.push(MOV(R9, Ptr("tohex_buffer")));
    asm.push(MOV(R10, Ptr("tohex_lut")));

    asm.label("tohex_top");
    asm.push(TEST(RCX, RCX));
    asm.push(JZ(Label("tohex_bottom")));
    asm.push(SUB(RCX, 4i8));

    asm.push(MOV(R11, RDI));
    asm.push(SHR(R11, CL));
    asm.push(AND(R11, 0x0f_i8));
    asm.push(MOV(R11B, Index(R11, R10)));
    asm.push(MOV(Indirect(R9), R11B));

    asm.push(INC(R9));
    asm.push(JMP(Label("tohex_top")));
    asm.label("tohex_bottom");

    asm.push(MOV(Indirect(R9), 0u8));
    asm.push(MOV(RAX, Ptr("tohex_buffer")));
    asm.push(RET);

    asm.label("terminal_callback");
    asm.push(RET);

    // Halt procedure
    asm.label("halt");
    asm.push(HLT);
    asm.push(JMP(Label("halt")));

    let code = asm.finish();

    let mut linker = ElfLinker::new();
    linker.add_segment(PF_R | PF_W, 1 << 12, data);
    linker.add_segment(PF_R | PF_X, 1 << 12, code);
    let linked = linker.finish();

    let mut file = File::create("kernel.elf")?;
    linked.write(&mut file)?;
    Ok(())
}
