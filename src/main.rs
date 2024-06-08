use std::{
    fmt::{self, format, Display},
    path::Path,
    process::Command,
    slice::from_raw_parts, io::Write,
};

use r8086::*;

fn main() {
    let path = "program";
    let program_image = std::fs::read(path).unwrap();
    println!("bits 16");

    let program_size = program_image.len() as u16;
    let mut register_file = RegisterFile::default();
    let mut memory = Memory::default();

    while register_file.ip < program_size {
        let end = (6 + register_file.ip).clamp(0, program_size);
        let bytes = &program_image[register_file.ip as usize..end as usize];
        let padded = &mut [0; 6];
        padded[0..bytes.len()].copy_from_slice(bytes);

        let instruction = decode_instruction(padded);
        // dbg!(instruction);
        println!("{instruction}");

        simulate(&mut register_file, &mut memory, instruction);
    }

    // dbg!(register_file);
    println!("ax: {:#06x}", register_file.ax);
    println!("bx: {:#06x}", register_file.bx);
    println!("cx: {:#06x}", register_file.cx);
    println!("dx: {:#06x}", register_file.dx);
    println!("sp: {:#06x}", register_file.sp);
    println!("bp: {:#06x}", register_file.bp);
    println!("si: {:#06x}", register_file.si);
    println!("di: {:#06x}", register_file.di);
    println!("flags: {:016b}", register_file.flags);
    println!("ip: {}", register_file.ip);

    println!("{:x?}", &memory.memory[1000..1010]);
    let mut file = std::fs::OpenOptions::new().write(true)
                             .create_new(true)
                             .open("memory.dump").unwrap();
    file.write_all(&memory.memory).unwrap();


}
