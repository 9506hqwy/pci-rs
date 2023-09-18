use super::parser;
use std::sync::OnceLock;

const PCIIDS: &str = include_str!("pciids/pci.ids");

static VENDORS: OnceLock<Vec<Vendor>> = OnceLock::new();

static CLASSES: OnceLock<Vec<BaseClass>> = OnceLock::new();

pub fn get_vendor(id: u16) -> Option<&'static Vendor> {
    VENDORS.get_or_init(init_vendor).iter().find(|v| v.id == id)
}

pub fn get_class(id: u8) -> Option<&'static BaseClass> {
    CLASSES.get_or_init(init_class).iter().find(|c| c.id == id)
}

fn init_vendor() -> Vec<Vendor> {
    let (v, _) = parser::parse(PCIIDS).unwrap();
    v
}

fn init_class() -> Vec<BaseClass> {
    let (_, c) = parser::parse(PCIIDS).unwrap();
    c
}

pub struct Vendor {
    id: u16,
    name: String,
    devices: Vec<Device>,
}

impl Vendor {
    pub fn new(id: u16, name: String, devices: Vec<Device>) -> Self {
        Vendor { id, name, devices }
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_device(&self, id: u16) -> Option<&Device> {
        self.devices.iter().find(|d| d.id == id)
    }
}

pub struct Device {
    id: u16,
    name: String,
    subsystems: Vec<SubSystem>,
}

impl Device {
    pub fn new(id: u16, name: String, subsystems: Vec<SubSystem>) -> Self {
        Device {
            id,
            name,
            subsystems,
        }
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_subsystem(&self, vendor: u16, device: u16) -> Option<&SubSystem> {
        self.subsystems
            .iter()
            .find(|s| s.sub_vendor == vendor && s.sub_device == device)
    }
}

pub struct SubSystem {
    sub_vendor: u16,
    sub_device: u16,
    name: String,
}

impl SubSystem {
    pub fn new(sub_vendor: u16, sub_device: u16, name: String) -> Self {
        SubSystem {
            sub_vendor,
            sub_device,
            name,
        }
    }

    pub fn sub_vendor(&self) -> u16 {
        self.sub_vendor
    }

    pub fn sub_device(&self) -> u16 {
        self.sub_device
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

pub struct BaseClass {
    id: u8,
    name: String,
    sub_classes: Vec<SubClass>,
}

impl BaseClass {
    pub fn new(id: u8, name: String, sub_classes: Vec<SubClass>) -> Self {
        BaseClass {
            id,
            name,
            sub_classes,
        }
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_sub_class(&self, id: u8) -> Option<&SubClass> {
        self.sub_classes.iter().find(|c| c.id == id)
    }
}

pub struct SubClass {
    id: u8,
    name: String,
    prog_ifs: Vec<ProgIf>,
}

impl SubClass {
    pub fn new(id: u8, name: String, prog_ifs: Vec<ProgIf>) -> Self {
        SubClass { id, name, prog_ifs }
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_prog_if(&self, id: u8) -> Option<&ProgIf> {
        self.prog_ifs.iter().find(|c| c.id == id)
    }
}

pub struct ProgIf {
    id: u8,
    name: String,
}

impl ProgIf {
    pub fn new(id: u8, name: String) -> Self {
        ProgIf { id, name }
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}
