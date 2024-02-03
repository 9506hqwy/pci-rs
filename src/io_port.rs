use super::error;
use super::Method;
use std::arch::asm;

const CONFIG_ADDRESS: u16 = 0x0CF8;
const CONFIG_DATA: u16 = 0x0CFC;

#[derive(Clone, Debug)]
pub struct IoPort {
    bus: u8,
    device: u8,
    func: u8,
}

impl Method for IoPort {
    fn try_from(bus: u8, device: u8, func: u8) -> Result<Self, error::Error> {
        Ok(IoPort { bus, device, func })
    }

    fn read8(&self, offset: u8) -> u8 {
        let (addr, shift) = multiple4(offset);
        let value = self.read32(addr);
        ((value >> shift) & 0x0000_00FF) as u8
    }

    fn read16(&self, offset: u8) -> u16 {
        let (addr, shift) = multiple4(offset);
        let value = self.read32(addr);
        ((value >> shift) & 0x0000_FFFF) as u16
    }

    fn read32(&self, offset: u8) -> u32 {
        set_config(self.bus, self.device, self.func, offset);
        read32(CONFIG_DATA)
    }
}

fn multiple4(value: u8) -> (u8, u8) {
    let r = value % 4;
    (value - r, r * 8)
}

fn set_config(bus: u8, device: u8, func: u8, offset: u8) {
    let mut config: u32 = 0;
    config |= offset as u32; // offset is only multiple of 4.
    config |= (func as u32) << 8;
    config |= (device as u32) << 11;
    config |= (bus as u32) << 16;
    config |= 0x8000_0000;
    write32(CONFIG_ADDRESS, config);
}

#[allow(dead_code)]
fn read8(address: u16) -> u8 {
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

#[allow(dead_code)]
fn write8(address: u16, value: u8) {
    unsafe {
        asm!(
            "outb %al, %dx",
            in("al") value,
            in("dx") address,
            options(att_syntax),
        );
    }
}

#[allow(dead_code)]
fn read16(address: u16) -> u16 {
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

#[allow(dead_code)]
fn write16(address: u16, value: u16) {
    unsafe {
        asm!(
            "outw %ax, %dx",
            in("ax") value,
            in("dx") address,
            options(att_syntax),
        );
    }
}

fn read32(address: u16) -> u32 {
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

fn write32(address: u16, value: u32) {
    unsafe {
        asm!(
            "outl %eax, %dx",
            in("eax") value,
            in("dx") address,
            options(att_syntax),
        );
    }
}
