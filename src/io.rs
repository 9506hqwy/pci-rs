use std::arch::asm;

pub fn read8(address: u16) -> u8 {
    let mut ret: u8;
    unsafe {
        asm!(
            "inb %dx, %al",
            out("al") ret,
            in("dx") address,
            options(att_syntax),
        );
    }
    ret
}

pub fn write8(address: u16, value: u8) {
    unsafe {
        asm!(
            "outb %al, %dx",
            in("al") value,
            in("dx") address,
            options(att_syntax),
        );
    }
}

pub fn read16(address: u16) -> u16 {
    let mut ret: u16;
    unsafe {
        asm!(
            "inw %dx, %ax",
            out("ax") ret,
            in("dx") address,
            options(att_syntax),
        );
    }
    ret
}

pub fn write16(address: u16, value: u16) {
    unsafe {
        asm!(
            "outw %ax, %dx",
            in("ax") value,
            in("dx") address,
            options(att_syntax),
        );
    }
}

pub fn read32(address: u16) -> u32 {
    let mut ret: u32;
    unsafe {
        asm!(
            "inl %dx, %eax",
            out("eax") ret,
            in("dx") address,
            options(att_syntax),
        );
    }
    ret
}

pub fn write32(address: u16, value: u32) {
    unsafe {
        asm!(
            "outl %eax, %dx",
            in("eax") value,
            in("dx") address,
            options(att_syntax),
        );
    }
}
