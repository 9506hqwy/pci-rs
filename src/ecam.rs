use super::error;
use super::Method;
use acpi::MemoryMappedConfiguration;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use libc;
use std::ffi::CString;
use std::io;
use std::os::raw::c_char;
use std::path::Path;
use std::ptr;
use std::slice;
use std::sync::OnceLock;

const MEM_DEV: &str = "/dev/mem";
const SIZE: usize = 64;

static MCFG: OnceLock<Option<MemoryMappedConfiguration>> = OnceLock::new();

pub fn support() -> bool {
    let mcfg = MCFG.get_or_init(init_mcfg).as_ref();
    let has_mem = Path::new(MEM_DEV).exists();
    mcfg.is_some() && has_mem
}

#[derive(Clone, Debug)]
pub struct Ecam {
    data: Bytes,
}

impl Method for Ecam {
    fn try_from(bus: u8, device: u8, func: u8) -> Result<Self, error::Error> {
        let data = read_mem(bus, device, func)?;
        Ok(Ecam { data })
    }

    fn read8(&self, offset: u8) -> u8 {
        let s = offset as usize;
        let e = s + 1;
        self.data.slice(s..e).get_u8()
    }

    fn read16(&self, offset: u8) -> u16 {
        let s = offset as usize;
        let e = s + 2;
        self.data.slice(s..e).get_u16_le()
    }

    fn read32(&self, offset: u8) -> u32 {
        let s = offset as usize;
        let e = s + 4;
        self.data.slice(s..e).get_u32_le()
    }
}

// -----------------------------------------------------------------------------------------------

fn init_mcfg() -> Option<MemoryMappedConfiguration> {
    acpi::get::<MemoryMappedConfiguration>("MCFG").ok()
}

fn read_mem(bus: u8, device: u8, func: u8) -> Result<Bytes, error::Error> {
    let offset = mem_offset(bus, device, func)?;
    let file = File::open_read(MEM_DEV)?;
    let mem = Memory::map(&file, offset)?;
    Ok(Bytes::from(mem))
}

fn mem_offset(bus: u8, device: u8, func: u8) -> Result<libc::off_t, error::Error> {
    let space = MCFG
        .get_or_init(init_mcfg)
        .as_ref()
        .and_then(|m| m.spaces.first());
    if space.is_none() {
        Err(error::Error::NotFoundAcpiMcfg)
    } else {
        let space = space.unwrap();

        let base_offset = i64::from_le_bytes(space.base_address);
        let offset = (((bus - space.bus_number_start) as i64) << 20)
            + ((device as i64) << 15)
            + ((func as i64) << 12)
            + base_offset;
        Ok(offset)
    }
}

// -----------------------------------------------------------------------------------------------

struct File {
    fd: libc::c_int,
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
        }
    }
}

impl File {
    fn open_read(path: &str) -> Result<File, io::Error> {
        let path = CString::new(path).unwrap();
        let fd = unsafe {
            libc::open(
                path.as_ptr() as *const c_char,
                libc::O_RDONLY | libc::O_DSYNC,
            )
        };

        if fd < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(File { fd })
        }
    }
}

// -----------------------------------------------------------------------------------------------

struct Memory {
    mem: *mut libc::c_void,
}

impl Drop for Memory {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.mem, SIZE);
        }
    }
}

impl From<Memory> for Bytes {
    fn from(value: Memory) -> Self {
        let mut bytes = BytesMut::with_capacity(SIZE);

        unsafe {
            for p in slice::from_raw_parts(value.mem as *const u8, SIZE) {
                bytes.put_u8(*p);
            }
        }

        bytes.freeze()
    }
}

impl Memory {
    fn map(file: &File, offset: libc::off_t) -> Result<Self, io::Error> {
        let mem = unsafe {
            libc::mmap(
                ptr::null_mut(),
                SIZE,
                libc::PROT_READ,
                libc::MAP_PRIVATE,
                file.fd,
                offset,
            )
        };

        if mem.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(Memory { mem })
        }
    }
}
