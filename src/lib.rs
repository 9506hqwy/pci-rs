pub mod ids;
pub mod io;

const CONFIG_ADDRESS: u16 = 0x0CF8;
const CONFIG_DATA: u16 = 0x0CFC;

const OFFSET_VENDOR_ID: u8 = 0x00;
const OFFSET_COMMAND: u8 = 0x04;
const OFFSET_REVISION_ID: u8 = 0x08;
const OFFSET_CACHE_LINE_SIZE: u8 = 0x0C;
const OFFSET_CAPABILITIES_POINTER: u8 = 0x34;
const OFFSET_INTERRUPT_LINE: u8 = 0x3C;

const OFFSET_TYPE1_BAR0: u8 = 0x10;
const OFFSET_TYPE1_BAR1: u8 = 0x14;
const OFFSET_TYPE1_PRIMARY_BUS_NUM: u8 = 0x18;

const NOT_USED: u16 = 0xFFFF;

pub fn get_pci_config(bus: u8, device: u8, func: u8) -> Option<PciConfig> {
    set_config(bus, device, func, OFFSET_VENDOR_ID);
    let value = io::read32(CONFIG_DATA);
    let vendor_id = extract_u16(value, 0);
    if vendor_id == NOT_USED {
        return None;
    }

    let device_id = extract_u16(value, 16);
    if device_id == NOT_USED {
        return None;
    }

    set_config(bus, device, func, OFFSET_COMMAND);
    let value = io::read32(CONFIG_DATA);
    let command = extract_u16(value, 0);
    let status = extract_u16(value, 16);

    set_config(bus, device, func, OFFSET_REVISION_ID);
    let value = io::read32(CONFIG_DATA);
    let revision_id = extract_u8(value, 0);
    let prog_if = extract_u8(value, 8);
    let sub_class = extract_u8(value, 16);
    let base_class = extract_u8(value, 24);

    set_config(bus, device, func, OFFSET_CACHE_LINE_SIZE);
    let value = io::read32(CONFIG_DATA);
    let cache_line_size = extract_u8(value, 0);
    let master_latency_timer = extract_u8(value, 8);
    let header_type = extract_u8(value, 16);
    let bist = extract_u8(value, 24);

    set_config(bus, device, func, OFFSET_CAPABILITIES_POINTER);
    let value = io::read32(CONFIG_DATA);
    let capabilities_pointer = extract_u8(value, 0);

    set_config(bus, device, func, OFFSET_INTERRUPT_LINE);
    let value = io::read32(CONFIG_DATA);
    let interrupt_line = extract_u8(value, 0);
    let interrupt_pin = extract_u8(value, 8);

    let config = PciConfig {
        slot: (bus, device, func),
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
pub struct PciConfig {
    slot: (u8, u8, u8),
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

impl PciConfig {
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

    pub fn get_type1_header(&self) -> Option<PciConfigType1> {
        if !self.header_type().type1() {
            return None;
        }

        set_config(self.slot.0, self.slot.1, self.slot.2, OFFSET_TYPE1_BAR0);
        let bar0 = io::read32(CONFIG_DATA);

        set_config(self.slot.0, self.slot.1, self.slot.2, OFFSET_TYPE1_BAR1);
        let bar1 = io::read32(CONFIG_DATA);

        set_config(
            self.slot.0,
            self.slot.1,
            self.slot.2,
            OFFSET_TYPE1_PRIMARY_BUS_NUM,
        );
        let value = io::read32(CONFIG_DATA);
        let primary_bus_number = extract_u8(value, 0);
        let secondary_bus_number = extract_u8(value, 8);
        let subordinate_bus_number = extract_u8(value, 16);
        let secondary_latency_timer = extract_u8(value, 24);

        let t1 = PciConfigType1 {
            bar0,
            bar1,
            primary_bus_number,
            secondary_bus_number,
            subordinate_bus_number,
            secondary_latency_timer,
        };

        Some(t1)
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
    pub fn type1(&self) -> bool {
        self.get_bool(0)
    }

    pub fn multi_functoin_device(&self) -> bool {
        self.get_bool(7)
    }

    fn get_bool(&self, bit: u8) -> bool {
        let mask: u8 = 1 << bit;
        (self.0 & mask) == mask
    }
}

fn extract_u8(value: u32, shift: u8) -> u8 {
    ((value >> shift) & 0x0000_00FF) as u8
}

fn extract_u16(value: u32, shift: u8) -> u16 {
    ((value >> shift) & 0x0000_FFFF) as u16
}

fn set_config(bus: u8, device: u8, func: u8, offset: u8) {
    let mut config: u32 = 0;
    config |= offset as u32; // offset is only multiple of 4.
    config |= (func as u32) << 8;
    config |= (device as u32) << 11;
    config |= (bus as u32) << 16;
    config |= 0x8000_0000;
    io::write32(CONFIG_ADDRESS, config);
}
