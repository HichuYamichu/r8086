use std::{
    fs::File,
    io::{Read, Write},
    process::Command,
};

use r8086::*;

fn main() {
    let input_asm_file_path = "input/program.asm";
    let input_bin_file_path = "input/program.bin";
    let output_asm_file_path = "output/disassembly.asm";
    let output_register_file_path = "output/register.txt";
    let memory_dump_path = "output/memory.dump";

    let nasm_status = Command::new("nasm")
        .arg("-f")
        .arg("bin")
        .arg("-o")
        .arg(input_bin_file_path)
        .arg(input_asm_file_path)
        .status()
        .expect("Failed to assemble.");

    if !nasm_status.success() {
        panic!("NASM failed to assemble the input file.");
    }

    let mut disassembly_output = std::fs::File::create(output_asm_file_path).unwrap();
    let mut register_output = std::fs::File::create(output_register_file_path).unwrap();
    let mut memory_dump = std::fs::File::create(memory_dump_path).unwrap();

    let mut input_bin_file = File::open(input_bin_file_path).unwrap();

    let mut register_file = RegisterFile::default();
    let mut memory = vec![0; 1024 * 1024];
    let program_size = input_bin_file.read(&mut memory[..]).unwrap() as u16;
    dbg!(&memory[0..program_size as _]);
    dbg!(program_size);

    writeln!(disassembly_output, "bits 16").unwrap();

    let mut offset = 0;
    while offset < program_size {
        let end = (offset + 6).clamp(0, program_size);
        let bytes = &memory[offset as usize..end as usize];
        let padded = &mut [0; 6];
        padded[0..bytes.len()].copy_from_slice(bytes);
        let instruction = decode_instruction(padded);
        offset += instruction.length as u16;
        writeln!(disassembly_output, "{instruction}").unwrap();
    }

    while register_file.ip < program_size {
        let end = (6 + register_file.ip).clamp(0, program_size);
        let bytes = &memory[register_file.ip as usize..end as usize];
        let padded = &mut [0; 6];
        padded[0..bytes.len()].copy_from_slice(bytes);

        let instruction = decode_instruction(padded);
        simulate(&mut register_file, &mut memory, instruction);
    }

    writeln!(register_output, "ax: {:#06x}", register_file.ax).unwrap();
    writeln!(register_output, "bx: {:#06x}", register_file.bx).unwrap();
    writeln!(register_output, "cx: {:#06x}", register_file.cx).unwrap();
    writeln!(register_output, "dx: {:#06x}", register_file.dx).unwrap();
    writeln!(register_output, "sp: {:#06x}", register_file.sp).unwrap();
    writeln!(register_output, "bp: {:#06x}", register_file.bp).unwrap();
    writeln!(register_output, "si: {:#06x}", register_file.si).unwrap();
    writeln!(register_output, "di: {:#06x}", register_file.di).unwrap();
    writeln!(register_output, "flags: {:016b}", register_file.flags).unwrap();
    writeln!(register_output, "ip: {}", register_file.ip).unwrap();

    memory_dump.write_all(&memory).unwrap();
}

