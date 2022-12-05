use std::{error::Error, fs::File};

use elf64::program::{PF_R, PF_W, PF_X};
use link::{ElfLinker, Label, Ptr, ReferenceFormat, Segment};
use x86::{
    address::*,
    instruction::*,
    register::{R16::*, R32::*, R64::*, R8::*},
};

pub mod elf64;
pub mod limine;
pub mod link;
pub mod math;
pub mod x86;

fn main() -> Result<(), Box<dyn Error>> {
    let mut rodata = Segment::new();
    rodata.align(8);

    rodata.offset_label(limine::RESPONSE_OFFSET, "terminal_response");
    rodata.append(&limine::Request::new(limine::TERMINAL_REQUEST, 0));
    rodata.append_reference("terminal_callback", ReferenceFormat::Abs64);

    rodata.offset_label(limine::RESPONSE_OFFSET, "bootloader_info_response");
    rodata.append(&limine::Request::new(limine::BOOTLOADER_INFO_REQUEST, 0));

    rodata.label("idtr");
    rodata.append(&64_u16.to_le_bytes()); // Limit
    rodata.append_reference("idt", ReferenceFormat::Abs64);

    rodata.label("str_hello");
    rodata.append(b"Hello \0");

    rodata.label("str_space");
    rodata.append(b" \0");

    rodata.label("str_newline");
    rodata.append(b"\n\0");

    rodata.label("str_oops");
    rodata.append(b"oops!\n\0");

    rodata.label("tohex_lut");
    rodata.append(b"0123456789abcdef");

    let mut data = Segment::new();

    data.label("idt");
    for _idt_index in 0..4 {
        // Offset 15..0
        data.append(&0u16.to_le_bytes());
        // Segment Selector
        // (segment 5, rpl 0, from limine-provided GDT)
        data.append(&(5u16 << 3).to_le_bytes());
        // Not present; RPL 0; Interrupt gate type
        data.append(&0x0e00_u16.to_le_bytes());
        // Offset 31..16
        data.append(&0u16.to_le_bytes());
        // Offset 63..32
        data.append(&0u32.to_le_bytes());
        // Reserved
        data.append(&0u32.to_le_bytes());
    }

    // TODO move to bss segment
    data.label("tohex_buffer");
    data.append(&[0u8; 32]);

    let mut asm = x86::Assembler::new();
    asm.label("code_start");

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

    asm.push(MOV(RSI, Ptr("str_newline")));
    asm.push(CALL(Label("print")));

    // Initialize IDT
    asm.push(MOV(RDI, Ptr("idt")));
    asm.push(MOV(RAX, Ptr("oops")));

    // 16 bytes per table entry; targeting INT3
    let gate_base: i8 = 16 * 3;
    // Offset 15..0
    asm.push(MOV(Index(RDI, gate_base), AX));
    // Offset 31..16
    asm.push(SHR(RAX, 16));
    asm.push(MOV(Index(RDI, gate_base + 6), AX));
    // Offset 63..32
    asm.push(SHR(RAX, 16));
    asm.push(MOV(Index(RDI, gate_base + 8), EAX));
    // Present
    asm.push(OR(Index(RDI, gate_base + 4), 0x8000_u16 as i16));

    asm.push(MOV(RAX, Ptr("idtr")));
    asm.push(LIDT(Indirect(RAX)));
    asm.push(STI);
    asm.push(NOP);
    asm.push(INT3);

    asm.push(MOV(RSI, Ptr("str_hello")));
    asm.push(CALL(Label("print")));

    asm.push(JMP(Label("halt")));

    asm.label("oops");
    asm.push(PUSH(RAX));
    asm.push(PUSH(RBX));
    asm.push(PUSH(RCX));
    asm.push(PUSH(RDX));
    asm.push(PUSH(RDI));
    asm.push(PUSH(RSI));
    asm.push(PUSH(R8));
    asm.push(PUSH(R9));
    asm.push(PUSH(R10));
    asm.push(PUSH(R11));

    asm.push(MOV(RSI, Ptr("str_oops")));
    asm.push(CALL(Label("print")));

    asm.push(POP(R11));
    asm.push(POP(R10));
    asm.push(POP(R9));
    asm.push(POP(R8));
    asm.push(POP(RSI));
    asm.push(POP(RDI));
    asm.push(POP(RDX));
    asm.push(POP(RCX));
    asm.push(POP(RBX));
    asm.push(POP(RAX));

    asm.push(STI);
    asm.push(IRET);

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
    linker.add_segment(PF_R, 1 << 12, rodata);
    linker.add_segment(PF_R | PF_W, 1 << 12, data);
    linker.add_segment(PF_R | PF_X, 1 << 12, code);
    let linked = linker.finish();

    let mut file = File::create("kernel.elf")?;
    linked.write(&mut file)?;
    Ok(())
}
