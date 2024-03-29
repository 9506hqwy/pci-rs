pub mod error;
pub mod ids;
pub mod io_port;
pub mod parser;

#[cfg(target_family = "unix")]
pub mod ecam;

#[cfg(target_family = "windows")]
pub mod cfgmgr32;

use std::rc::Rc;

pub const OFFSET_VENDOR_ID: u8 = 0x00;
pub const OFFSET_DEVICE_ID: u8 = 0x02;
pub const OFFSET_COMMAND: u8 = 0x04;
pub const OFFSET_STATUS: u8 = 0x06;
pub const OFFSET_REVISION_ID: u8 = 0x08;
pub const OFFSET_PROG_INTF: u8 = 0x09;
pub const OFFSET_SUB_CLASS: u8 = 0x0A;
pub const OFFSET_BASE_CLASS: u8 = 0x0B;
pub const OFFSET_CACHE_LINE_SIZE: u8 = 0x0C;
pub const OFFSET_MASTER_LATENCY_TIMER: u8 = 0x0D;
pub const OFFSET_HEADER_TYPE: u8 = 0x0E;
pub const OFFSET_BIST: u8 = 0x0F;
pub const OFFSET_CAPABILITIES_POINTER: u8 = 0x34;
pub const OFFSET_INTERRUPT_LINE: u8 = 0x3C;
pub const OFFSET_INTERRUPT_PIN: u8 = 0x3D;

pub const OFFSET_TYPE0_BAR0: u8 = 0x10;
pub const OFFSET_TYPE0_BAR1: u8 = 0x14;
pub const OFFSET_TYPE0_BAR2: u8 = 0x18;
pub const OFFSET_TYPE0_BAR3: u8 = 0x1C;
pub const OFFSET_TYPE0_BAR4: u8 = 0x20;
pub const OFFSET_TYPE0_BAR5: u8 = 0x24;
pub const OFFSET_TYPE0_CARDBUS: u8 = 0x28;
pub const OFFSET_TYPE0_SUBSYSTEM_VENDOR_ID: u8 = 0x2C;
pub const OFFSET_TYPE0_SUBSYSTEM_ID: u8 = 0x2E;
pub const OFFSET_TYPE0_EXPANSION: u8 = 0x30;

pub const OFFSET_TYPE1_BAR0: u8 = 0x10;
pub const OFFSET_TYPE1_BAR1: u8 = 0x14;
pub const OFFSET_TYPE1_PRIMARY_BUS_NUM: u8 = 0x18;
pub const OFFSET_TYPE1_SECONDARY_BUS_NUM: u8 = 0x19;
pub const OFFSET_TYPE1_SUBORDINATE_BUS_NUM: u8 = 0x1A;
pub const OFFSET_TYPE1_SECONDARY_LATENCY_TIMER: u8 = 0x1B;
pub const OFFSET_TYPE1_EXPANSION: u8 = 0x38;

pub const OFFSET_BAR_TYPE_MASK: u32 = 0x01;
pub const OFFSET_BAR_TYPE_IO: u32 = 0x01;

pub const OFFSET_BAR_ADDRSPACE_MASK: u32 = 0x06;
pub const OFFSET_BAR_ADDRSPACE_16BIT: u32 = 0x02;
pub const OFFSET_BAR_ADDRSPACE_32BIT: u32 = 0x00;
pub const OFFSET_BAR_ADDRSPACE_64BIT: u32 = 0x04;

pub const OFFSET_BAR_PREFETCH_MASK: u32 = 0x08;
pub const OFFSET_BAR_PREFETCH_ENABLE: u32 = 0x08;

pub const NOT_USED: u16 = 0xFFFF;

#[derive(Debug)]
#[repr(u8)]
pub enum CapabilityId {
    Null = 0x00,
    Pm = 0x01,
    Agp = 0x02,
    Vpd = 0x03,
    SlotId = 0x04,
    Msi = 0x05,
    CompatPciHotSwap = 0x06,
    PciX = 0x07,
    HyperTransport = 0x08,
    VendorSpecific = 0x09,
    DebugPort = 0x0A,
    CompatPciCrc = 0x0B,
    PciHotPlug = 0x0C,
    PciBridgeSubVendorId = 0x0D,
    Agp8x = 0x0E,
    SecureDevice = 0x0F,
    PciE = 0x10,
    MsiX = 0x11,
    SataConfig = 0x12,
    AdvanedFeature = 0x13,
    EnhancedAllocation = 0x14,
    FlatteningPortalBridge = 0x15,
}

pub fn get_pci_config<T: Method>(method: T) -> Option<PciConfig<T>> {
    let vendor_id = method.read16(OFFSET_VENDOR_ID);
    if vendor_id == NOT_USED {
        return None;
    }

    let device_id = method.read16(OFFSET_DEVICE_ID);
    if device_id == NOT_USED {
        return None;
    }

    let command = method.read16(OFFSET_COMMAND);
    let status = method.read16(OFFSET_STATUS);
    let revision_id = method.read8(OFFSET_REVISION_ID);
    let prog_if = method.read8(OFFSET_PROG_INTF);
    let sub_class = method.read8(OFFSET_SUB_CLASS);
    let base_class = method.read8(OFFSET_BASE_CLASS);
    let cache_line_size = method.read8(OFFSET_CACHE_LINE_SIZE);
    let master_latency_timer = method.read8(OFFSET_MASTER_LATENCY_TIMER);
    let header_type = method.read8(OFFSET_HEADER_TYPE);
    let bist = method.read8(OFFSET_BIST);
    let capabilities_pointer = method.read8(OFFSET_CAPABILITIES_POINTER);
    let interrupt_line = method.read8(OFFSET_INTERRUPT_LINE);
    let interrupt_pin = method.read8(OFFSET_INTERRUPT_PIN);

    let config = PciConfig {
        method: Rc::new(method),
        vendor_id,
        device_id,
        command: Command(command),
        status: Status(status),
        revision_id,
        class_code: ClassCode(base_class, sub_class, prog_if),
        cache_line_size,
        master_latency_timer,
        header_type: HeaderType(header_type),
        bist,
        capabilities_pointer,
        interrupt_line,
        interrupt_pin,
    };

    Some(config)
}

#[derive(Clone, Debug)]
pub struct PciConfig<T: Method> {
    method: Rc<T>,
    vendor_id: u16,
    device_id: u16,
    command: Command,
    status: Status,
    revision_id: u8,
    class_code: ClassCode,
    cache_line_size: u8,
    master_latency_timer: u8,
    header_type: HeaderType,
    bist: u8,
    capabilities_pointer: u8,
    interrupt_line: u8,
    interrupt_pin: u8,
}

impl<T: Method> PciConfig<T> {
    pub fn vendor_id(&self) -> u16 {
        self.vendor_id
    }

    pub fn device_id(&self) -> u16 {
        self.device_id
    }

    pub fn command(&self) -> Command {
        self.command
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn revision_id(&self) -> u8 {
        self.revision_id
    }

    pub fn class_code(&self) -> ClassCode {
        self.class_code
    }

    pub fn cache_line_size(&self) -> u8 {
        self.cache_line_size
    }

    pub fn master_latency_timer(&self) -> u8 {
        self.master_latency_timer
    }

    pub fn header_type(&self) -> HeaderType {
        self.header_type
    }

    pub fn bist(&self) -> u8 {
        self.bist
    }

    pub fn capabilities_pointer(&self) -> u8 {
        self.capabilities_pointer
    }

    pub fn interrupt_line(&self) -> u8 {
        self.interrupt_line
    }

    pub fn interrupt_pin(&self) -> u8 {
        self.interrupt_pin
    }

    pub fn get_type0_header(&self) -> Option<PciConfigType0> {
        if !self.header_type().type0() {
            return None;
        }

        let bar0 = self.method.read32(OFFSET_TYPE0_BAR0);
        let bar1 = self.method.read32(OFFSET_TYPE0_BAR1);
        let bar2 = self.method.read32(OFFSET_TYPE0_BAR2);
        let bar3 = self.method.read32(OFFSET_TYPE0_BAR3);
        let bar4 = self.method.read32(OFFSET_TYPE0_BAR4);
        let bar5 = self.method.read32(OFFSET_TYPE0_BAR5);
        let cardbus_cis_pointer = self.method.read32(OFFSET_TYPE0_CARDBUS);
        let subsystem_vendor_id = self.method.read16(OFFSET_TYPE0_SUBSYSTEM_VENDOR_ID);
        let subsystem_id = self.method.read16(OFFSET_TYPE0_SUBSYSTEM_ID);
        let expansion_rom = self.method.read32(OFFSET_TYPE0_EXPANSION);

        let t0 = PciConfigType0 {
            bar0,
            bar1,
            bar2,
            bar3,
            bar4,
            bar5,
            cardbus_cis_pointer,
            subsystem_vendor_id,
            subsystem_id,
            expansion_rom,
        };

        Some(t0)
    }

    pub fn get_type1_header(&self) -> Option<PciConfigType1> {
        if !self.header_type().type1() {
            return None;
        }

        let bar0 = self.method.read32(OFFSET_TYPE1_BAR0);
        let bar1 = self.method.read32(OFFSET_TYPE1_BAR1);
        let primary_bus_number = self.method.read8(OFFSET_TYPE1_PRIMARY_BUS_NUM);
        let secondary_bus_number = self.method.read8(OFFSET_TYPE1_SECONDARY_BUS_NUM);
        let subordinate_bus_number = self.method.read8(OFFSET_TYPE1_SUBORDINATE_BUS_NUM);
        let secondary_latency_timer = self.method.read8(OFFSET_TYPE1_SECONDARY_LATENCY_TIMER);
        let expansion_rom = self.method.read32(OFFSET_TYPE1_EXPANSION);

        let t1 = PciConfigType1 {
            bar0,
            bar1,
            primary_bus_number,
            secondary_bus_number,
            subordinate_bus_number,
            secondary_latency_timer,
            expansion_rom,
        };

        Some(t1)
    }

    pub fn capability(&self) -> Option<PciCapability<T>> {
        let value = (self.capabilities_pointer as u32) << 8;
        let cap = PciCapability::from(self.method.clone(), value);
        cap.next()
    }
}

#[derive(Clone, Debug)]
pub struct PciConfigType0 {
    bar0: u32,
    bar1: u32,
    bar2: u32,
    bar3: u32,
    bar4: u32,
    bar5: u32,
    cardbus_cis_pointer: u32,
    subsystem_vendor_id: u16,
    subsystem_id: u16,
    expansion_rom: u32,
    // TODO:
}

impl PciConfigType0 {
    pub fn bar0(&self) -> u32 {
        self.bar0
    }

    pub fn bar1(&self) -> u32 {
        self.bar1
    }

    pub fn bar2(&self) -> u32 {
        self.bar2
    }

    pub fn bar3(&self) -> u32 {
        self.bar3
    }

    pub fn bar4(&self) -> u32 {
        self.bar4
    }

    pub fn bar5(&self) -> u32 {
        self.bar5
    }

    pub fn cardbus_cis_pointer(&self) -> u32 {
        self.cardbus_cis_pointer
    }

    pub fn subsystem_vendor_id(&self) -> u16 {
        self.subsystem_vendor_id
    }

    pub fn subsystem_id(&self) -> u16 {
        self.subsystem_id
    }

    pub fn expansion_rom(&self) -> u32 {
        self.expansion_rom
    }

    pub fn bars(&self) -> Vec<PciBaseAddress> {
        let mut addrs = vec![];

        let addr = PciBaseAddress::from(self.bar0, self.bar1);
        let mut skip = addr.b64();
        addrs.push(addr);

        if !skip {
            let addr = PciBaseAddress::from(self.bar1, self.bar2);
            skip = addr.b64();
            addrs.push(addr);
        } else {
            skip = false;
        }

        if !skip {
            let addr = PciBaseAddress::from(self.bar2, self.bar3);
            skip = addr.b64();
            addrs.push(addr);
        } else {
            skip = false;
        }

        if !skip {
            let addr = PciBaseAddress::from(self.bar3, self.bar4);
            skip = addr.b64();
            addrs.push(addr);
        } else {
            skip = false;
        }

        if !skip {
            let addr = PciBaseAddress::from(self.bar4, self.bar5);
            skip = addr.b64();
            addrs.push(addr);
        } else {
            skip = false;
        }

        if !skip {
            let addr = PciBaseAddress::from(self.bar5, 0);
            addrs.push(addr);
        }

        addrs
    }
}

#[derive(Clone, Debug)]
pub struct PciConfigType1 {
    bar0: u32,
    bar1: u32,
    primary_bus_number: u8,
    secondary_bus_number: u8,
    subordinate_bus_number: u8,
    secondary_latency_timer: u8,
    expansion_rom: u32,
    // TODO:
}

impl PciConfigType1 {
    pub fn bar0(&self) -> u32 {
        self.bar0
    }

    pub fn bar1(&self) -> u32 {
        self.bar1
    }

    pub fn parity_error_response(&self) -> u8 {
        self.primary_bus_number
    }

    pub fn secondary_bus_number(&self) -> u8 {
        self.secondary_bus_number
    }

    pub fn subordinate_bus_number(&self) -> u8 {
        self.subordinate_bus_number
    }

    pub fn secondary_latency_timer(&self) -> u8 {
        self.secondary_latency_timer
    }

    pub fn expansion_rom(&self) -> u32 {
        self.expansion_rom
    }
}

#[derive(Clone, Debug, Default)]
pub struct PciBaseAddress {
    bar: u64,
    io_space: bool,
    b16: bool,
    b32: bool,
    b64: bool,
    prefetchable: bool,
}

impl PciBaseAddress {
    pub fn from(bar: u32, nbar: u32) -> Self {
        let mut addr = PciBaseAddress::default();

        if (bar & OFFSET_BAR_TYPE_MASK) == OFFSET_BAR_TYPE_IO {
            addr.bar = (bar & 0xFFFF_FFFC) as u64;
            addr.io_space = true;
            addr
        } else {
            addr.bar = if (bar & OFFSET_BAR_ADDRSPACE_MASK) == OFFSET_BAR_ADDRSPACE_64BIT {
                ((nbar as u64) << 32) + ((bar & 0xFFFF_FFF0) as u64)
            } else {
                (bar & 0xFFFF_FFF0) as u64
            };
            addr.b16 = (bar & OFFSET_BAR_ADDRSPACE_MASK) == OFFSET_BAR_ADDRSPACE_16BIT;
            addr.b32 = (bar & OFFSET_BAR_ADDRSPACE_MASK) == OFFSET_BAR_ADDRSPACE_32BIT;
            addr.b64 = (bar & OFFSET_BAR_ADDRSPACE_MASK) == OFFSET_BAR_ADDRSPACE_64BIT;
            addr.prefetchable = (bar & OFFSET_BAR_PREFETCH_MASK) == OFFSET_BAR_PREFETCH_ENABLE;
            addr
        }
    }

    pub fn bar(&self) -> u64 {
        self.bar
    }

    pub fn io_space(&self) -> bool {
        self.io_space
    }

    pub fn b16(&self) -> bool {
        self.b16
    }

    pub fn b32(&self) -> bool {
        self.b32
    }

    pub fn b64(&self) -> bool {
        self.b64
    }

    pub fn prefetchable(&self) -> bool {
        self.prefetchable
    }
}

#[derive(Clone, Debug)]
pub struct PciCapability<T: Method> {
    method: Rc<T>,
    id: u8,
    next_pointer: u8,
}

impl<T: Method> PciCapability<T> {
    pub fn from(method: Rc<T>, value: u32) -> Self {
        PciCapability {
            method,
            id: (value & 0x0000_00FF) as u8,
            next_pointer: ((value & 0x0000_FF00) >> 8) as u8,
        }
    }

    pub fn id(&self) -> Option<CapabilityId> {
        match self.id {
            0x00 => Some(CapabilityId::Null),
            0x01 => Some(CapabilityId::Pm),
            0x02 => Some(CapabilityId::Agp),
            0x03 => Some(CapabilityId::Vpd),
            0x04 => Some(CapabilityId::SlotId),
            0x05 => Some(CapabilityId::Msi),
            0x06 => Some(CapabilityId::CompatPciHotSwap),
            0x07 => Some(CapabilityId::PciX),
            0x08 => Some(CapabilityId::HyperTransport),
            0x09 => Some(CapabilityId::VendorSpecific),
            0x0A => Some(CapabilityId::DebugPort),
            0x0B => Some(CapabilityId::CompatPciCrc),
            0x0C => Some(CapabilityId::PciHotPlug),
            0x0D => Some(CapabilityId::PciBridgeSubVendorId),
            0x0E => Some(CapabilityId::Agp8x),
            0x0F => Some(CapabilityId::SecureDevice),
            0x10 => Some(CapabilityId::PciE),
            0x11 => Some(CapabilityId::MsiX),
            0x12 => Some(CapabilityId::SataConfig),
            0x13 => Some(CapabilityId::AdvanedFeature),
            0x14 => Some(CapabilityId::EnhancedAllocation),
            0x15 => Some(CapabilityId::FlatteningPortalBridge),
            _ => None,
        }
    }

    pub fn next_pointer(&self) -> u8 {
        self.next_pointer
    }

    pub fn next(&self) -> Option<PciCapability<T>> {
        if self.next_pointer == 0 {
            None
        } else {
            let data = self.method.read32(self.next_pointer);
            Some(PciCapability::from(self.method.clone(), data))
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Command(u16);

impl Command {
    pub fn io_space_enable(&self) -> bool {
        self.get_bool(0)
    }

    pub fn memory_space_enable(&self) -> bool {
        self.get_bool(1)
    }

    pub fn bus_master_enable(&self) -> bool {
        self.get_bool(2)
    }

    pub fn special_cycle_enable(&self) -> bool {
        self.get_bool(3)
    }

    pub fn memory_write_and_invalidate(&self) -> bool {
        self.get_bool(4)
    }

    pub fn vga_palette_snoop(&self) -> bool {
        self.get_bool(5)
    }

    pub fn parity_error_response(&self) -> bool {
        self.get_bool(6)
    }

    pub fn idsel_stepping_wait_cycle_control(&self) -> bool {
        self.get_bool(7)
    }

    pub fn serr_enable(&self) -> bool {
        self.get_bool(8)
    }

    pub fn fast_back_to_back_transactions_enable(&self) -> bool {
        self.get_bool(9)
    }

    pub fn interrupt_disable(&self) -> bool {
        self.get_bool(10)
    }

    fn get_bool(&self, bit: u8) -> bool {
        let mask: u16 = 1 << bit;
        (self.0 & mask) == mask
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Status(u16);

impl Status {
    pub fn interrupt_status(&self) -> bool {
        self.get_bool(3)
    }

    pub fn capabilities_list(&self) -> bool {
        self.get_bool(4)
    }

    pub fn mhz_66_capable(&self) -> bool {
        self.get_bool(5)
    }

    pub fn user_definable_features(&self) -> bool {
        self.get_bool(6)
    }

    pub fn fast_back_to_back_transactions_capable(&self) -> bool {
        self.get_bool(7)
    }

    pub fn master_data_parity_error(&self) -> bool {
        self.get_bool(8)
    }

    pub fn devsel_timing(&self) -> u8 {
        ((self.0 >> 9) & 0x0003) as u8
    }

    pub fn signaled_target_abort(&self) -> bool {
        self.get_bool(11)
    }

    pub fn received_target_abort(&self) -> bool {
        self.get_bool(12)
    }

    pub fn received_master_abort(&self) -> bool {
        self.get_bool(13)
    }

    pub fn signaled_system_error(&self) -> bool {
        self.get_bool(14)
    }

    pub fn detected_parity_error(&self) -> bool {
        self.get_bool(15)
    }

    fn get_bool(&self, bit: u8) -> bool {
        let mask: u16 = 1 << bit;
        (self.0 & mask) == mask
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ClassCode(u8, u8, u8);

impl ClassCode {
    pub fn base_class(&self) -> u8 {
        self.0
    }

    pub fn sub_class(&self) -> u8 {
        self.1
    }

    pub fn prog_if(&self) -> u8 {
        self.2
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HeaderType(u8);

impl HeaderType {
    pub fn type0(&self) -> bool {
        self.get_type() == 0
    }

    pub fn type1(&self) -> bool {
        self.get_type() == 1
    }

    pub fn multi_function_device(&self) -> bool {
        self.get_bool(7)
    }

    fn get_bool(&self, bit: u8) -> bool {
        let mask: u8 = 1 << bit;
        (self.0 & mask) == mask
    }

    fn get_type(&self) -> u8 {
        self.0 & 0x7F
    }
}

pub trait Method: Sized + Clone {
    fn try_from(bus: u8, device: u8, func: u8) -> Result<Self, error::Error>;

    fn read8(&self, offset: u8) -> u8;

    fn read16(&self, offset: u8) -> u16;

    fn read32(&self, offset: u8) -> u32;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;

    #[test]
    fn parse() {
        let mut f = File::open("src/pciids/pci.ids").unwrap();
        let mut content = String::new();
        f.read_to_string(&mut content).unwrap();

        let (v, c) = parser::parse(&content).unwrap();
        assert!(!v.is_empty());
        assert!(!c.is_empty());
    }
}
