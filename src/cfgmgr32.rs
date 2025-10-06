use super::{
    Method, NOT_USED, OFFSET_BASE_CLASS, OFFSET_BIST, OFFSET_CACHE_LINE_SIZE,
    OFFSET_CAPABILITIES_POINTER, OFFSET_COMMAND, OFFSET_DEVICE_ID, OFFSET_HEADER_TYPE,
    OFFSET_INTERRUPT_LINE, OFFSET_INTERRUPT_PIN, OFFSET_MASTER_LATENCY_TIMER, OFFSET_PROG_INTF,
    OFFSET_REVISION_ID, OFFSET_STATUS, OFFSET_SUB_CLASS, OFFSET_TYPE0_BAR0, OFFSET_TYPE0_BAR1,
    OFFSET_TYPE0_BAR2, OFFSET_TYPE0_BAR3, OFFSET_TYPE0_BAR4, OFFSET_TYPE0_BAR5,
    OFFSET_TYPE0_CARDBUS, OFFSET_TYPE0_EXPANSION, OFFSET_TYPE0_SUBSYSTEM_ID,
    OFFSET_TYPE0_SUBSYSTEM_VENDOR_ID, OFFSET_TYPE1_EXPANSION, OFFSET_TYPE1_PRIMARY_BUS_NUM,
    OFFSET_TYPE1_SECONDARY_BUS_NUM, OFFSET_TYPE1_SECONDARY_LATENCY_TIMER,
    OFFSET_TYPE1_SUBORDINATE_BUS_NUM, OFFSET_VENDOR_ID, error,
};
use bytes::{Buf, Bytes};
use std::sync::OnceLock;
use windows::Win32::Devices::DeviceAndDriverInstallation::{
    CM_GETIDLIST_FILTER_ENUMERATOR, CM_GETIDLIST_FILTER_PRESENT, CM_Get_DevNode_PropertyW,
    CM_Get_Device_ID_List_SizeA, CM_Get_Device_ID_ListA, CM_LOCATE_DEVNODE_NORMAL,
    CM_Locate_DevNodeA, CM_MapCrToWin32Err, CR_BUFFER_SMALL, CR_SUCCESS,
};
use windows::Win32::Devices::FunctionDiscovery::{
    PKEY_Device_Address, PKEY_Device_BusNumber, PKEY_Device_BusRelations, PKEY_Device_HardwareIds,
};
use windows::Win32::Devices::Properties::DEVPROPTYPE;
use windows::Win32::Foundation::{DEVPROPKEY, ERROR_INVALID_DATA, PROPERTYKEY, WIN32_ERROR};
use windows::Win32::Globalization::{CP_ACP, WC_COMPOSITECHECK, WideCharToMultiByte};
use windows::core::{Error, PCSTR};

static IDS: OnceLock<Vec<DevNode>> = OnceLock::new();

pub fn support() -> bool {
    !get_dev_nodes().is_empty()
}

#[derive(Clone, Debug)]
pub struct Cfgmgr32 {
    node: DevNode,
}

impl Method for Cfgmgr32 {
    fn try_from(bus: u8, device: u8, func: u8) -> Result<Self, error::Error> {
        let node = get_dev_nodes()
            .iter()
            .find(|n| n.bus == bus && n.device == device && n.func == func)
            .cloned()
            .unwrap_or_default();
        Ok(Cfgmgr32 { node })
    }

    fn read8(&self, offset: u8) -> u8 {
        match offset {
            OFFSET_REVISION_ID => self.get_revision(),
            OFFSET_PROG_INTF => self.get_prog_intf(),
            OFFSET_SUB_CLASS => self.get_sub_class(),
            OFFSET_BASE_CLASS => self.get_base_class(),
            OFFSET_CACHE_LINE_SIZE => 0,
            OFFSET_MASTER_LATENCY_TIMER => 0,
            OFFSET_HEADER_TYPE => self.get_header(),
            OFFSET_BIST => 0,
            OFFSET_CAPABILITIES_POINTER => 0,
            OFFSET_INTERRUPT_LINE => 0,
            OFFSET_INTERRUPT_PIN => 0,
            OFFSET_TYPE1_PRIMARY_BUS_NUM => self.get_primary_bus_number(),
            OFFSET_TYPE1_SECONDARY_BUS_NUM => self.get_secondary_bus_number(),
            OFFSET_TYPE1_SUBORDINATE_BUS_NUM => 0,
            OFFSET_TYPE1_SECONDARY_LATENCY_TIMER => 0,
            _ => unimplemented!(),
        }
    }

    fn read16(&self, offset: u8) -> u16 {
        match offset {
            OFFSET_VENDOR_ID => self.get_vendor(),
            OFFSET_DEVICE_ID => self.get_device(),
            OFFSET_COMMAND => 0,
            OFFSET_STATUS => 0,
            OFFSET_TYPE0_SUBSYSTEM_VENDOR_ID => self.get_subsys_id(),
            OFFSET_TYPE0_SUBSYSTEM_ID => self.get_subsys_vendor(),
            _ => unimplemented!(),
        }
    }

    fn read32(&self, offset: u8) -> u32 {
        match offset {
            OFFSET_TYPE0_BAR0 => 0,
            OFFSET_TYPE0_BAR1 => 0,
            OFFSET_TYPE0_BAR2 => 0,
            OFFSET_TYPE0_BAR3 => 0,
            OFFSET_TYPE0_BAR4 => 0,
            OFFSET_TYPE0_BAR5 => 0,
            OFFSET_TYPE0_CARDBUS => 0,
            OFFSET_TYPE0_EXPANSION => 0,
            //OFFSET_TYPE1_BAR0 => 0,
            //OFFSET_TYPE1_BAR1 => 0,
            OFFSET_TYPE1_EXPANSION => 0,
            _ => unimplemented!(),
        }
    }
}

impl Cfgmgr32 {
    fn get_vendor(&self) -> u16 {
        self.node.components.ven
    }

    fn get_device(&self) -> u16 {
        self.node.components.dev
    }

    fn get_revision(&self) -> u8 {
        self.node.components.rev
    }

    fn get_base_class(&self) -> u8 {
        self.node.components.base_class
    }

    fn get_sub_class(&self) -> u8 {
        self.node.components.sub_class
    }

    fn get_prog_intf(&self) -> u8 {
        self.node.components.prog_intf
    }

    fn get_subsys_id(&self) -> u16 {
        self.node.components.subsys_id
    }

    fn get_subsys_vendor(&self) -> u16 {
        self.node.components.subsys_ven
    }

    fn get_header(&self) -> u8 {
        let mut header = 0x80; // Multi Function Device

        if self.is_type1() {
            header |= 0x01; // type1;
        }

        header
    }

    fn get_primary_bus_number(&self) -> u8 {
        self.node.bus
    }

    fn get_secondary_bus_number(&self) -> u8 {
        self.pci_child_ids()
            .and_then(|ids| {
                ids.first()
                    .map(|id| get_dev_nodes().iter().find(|n| &n.id == id).unwrap().bus)
            })
            .unwrap_or(0xFF)
    }

    fn is_pci_bridge(&self) -> bool {
        self.get_base_class() == 0x06 && self.get_sub_class() == 0x04
    }

    fn is_type1(&self) -> bool {
        self.node.child_ids.is_some() || self.is_pci_bridge()
    }

    fn pci_child_ids(&self) -> Option<Vec<String>> {
        self.node.child_ids.as_ref().map(|ids| {
            ids.iter()
                .filter(|i| i.starts_with("PCI\\"))
                .cloned()
                .collect()
        })
    }
}

fn get_dev_nodes() -> &'static [DevNode] {
    IDS.get_or_init(init_dev_node).as_ref()
}

fn init_dev_node() -> Vec<DevNode> {
    let ids = get_device_ids("PCI").unwrap();
    ids.iter()
        .map(|i| DevNode::try_from(i.as_str()).unwrap())
        .collect()
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct DevNode {
    bus: u8,
    device: u8,
    func: u8,
    id: String,
    child_ids: Option<Vec<String>>,
    components: Components,
}

impl Default for DevNode {
    fn default() -> Self {
        DevNode {
            bus: 0xFF,
            device: 0xFF,
            func: 0xFF,
            id: String::default(),
            child_ids: None,
            components: Components::default(),
        }
    }
}

impl TryFrom<&str> for DevNode {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let devinst = get_device_inst(value)?;
        let bus = get_device_bus_number(devinst)?;
        let (device, func) = get_device_address(devinst)?;
        let hardware_ids = get_device_hardware_ids(devinst)?;
        let child_ids = get_device_bus_relations(devinst).ok();

        Ok(DevNode {
            bus: bus as u8,
            device: device as u8,
            func: func as u8,
            id: value.to_string(),
            child_ids,
            components: Components::from(hardware_ids.as_slice()),
        })
    }
}

// -----------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct Components {
    ven: u16,
    dev: u16,
    subsys_id: u16,
    subsys_ven: u16,
    rev: u8,
    base_class: u8,
    sub_class: u8,
    prog_intf: u8,
}

impl Default for Components {
    fn default() -> Self {
        Components {
            ven: NOT_USED,
            dev: NOT_USED,
            subsys_id: 0,
            subsys_ven: 0,
            rev: 0,
            base_class: 0,
            sub_class: 0,
            prog_intf: 0,
        }
    }
}

impl From<&[String]> for Components {
    fn from(values: &[String]) -> Self {
        // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/identifiers-for-pci-devices
        let mut container = Components::default();

        for value in values {
            for component in value.strip_prefix("PCI\\").unwrap().split('&') {
                if let Some(ven) = component.strip_prefix("VEN_") {
                    container.ven = u16::from_str_radix(ven, 16).unwrap();
                } else if let Some(dev) = component.strip_prefix("DEV_") {
                    container.dev = u16::from_str_radix(dev, 16).unwrap();
                } else if let Some(subsys) = component.strip_prefix("SUBSYS_") {
                    container.subsys_id = u16::from_str_radix(&subsys[0..4], 16).unwrap();
                    container.subsys_ven = u16::from_str_radix(&subsys[4..8], 16).unwrap();
                } else if let Some(rev) = component.strip_prefix("REV_") {
                    container.rev = u8::from_str_radix(&rev[0..2], 16).unwrap();
                } else if let Some(cc) = component.strip_prefix("CC_") {
                    container.base_class = u8::from_str_radix(&cc[0..2], 16).unwrap();
                    container.sub_class = u8::from_str_radix(&cc[2..4], 16).unwrap();
                    if cc.len() > 4 {
                        container.prog_intf = u8::from_str_radix(&cc[4..6], 16).unwrap();
                    }
                }
            }
        }

        container
    }
}

// -----------------------------------------------------------------------------------------------

fn get_device_address(devinst: u32) -> Result<(u16, u16), Error> {
    // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devpkey-device-address
    let (mut bytes, _) = get_device_prop(devinst, &PKEY_Device_Address)?;
    let address = bytes.get_u32_le();
    // https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-iogetdeviceproperty#devicepropertyaddress
    let device = (address >> 16) as u16;
    let func = (address & 0x0000_FFFF) as u16;
    Ok((device, func))
}

fn get_device_bus_number(devinst: u32) -> Result<u32, Error> {
    // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devpkey-device-busnumber
    let (mut bytes, _) = get_device_prop(devinst, &PKEY_Device_BusNumber)?;
    Ok(bytes.get_u32_le())
}

fn get_device_bus_relations(devinst: u32) -> Result<Vec<String>, Error> {
    // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devpkey-device-busrelations
    get_device_prop(devinst, &PKEY_Device_BusRelations).and_then(|(mut bytes_wide, _)| {
        let bytes_mb = wide_to_multi(&mut bytes_wide)?;
        Ok(multi_sz(&bytes_mb))
    })
}

fn get_device_hardware_ids(devinst: u32) -> Result<Vec<String>, Error> {
    // https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devpkey-device-hardwareids
    let (mut bytes_wide, _) = get_device_prop(devinst, &PKEY_Device_HardwareIds)?;
    let bytes_mb = wide_to_multi(&mut bytes_wide)?;
    Ok(multi_sz(&bytes_mb))
}

// -----------------------------------------------------------------------------------------------

fn multi_sz(value: &[u8]) -> Vec<String> {
    value
        .split(|&c| c == 0)
        .map(|i| String::from_utf8_lossy(i))
        .filter(|i| !i.is_empty())
        .map(|i| i.to_string())
        .collect()
}

// -----------------------------------------------------------------------------------------------

fn get_device_ids(filter: &str) -> Result<Vec<String>, Error> {
    let mut size: u32 = 0;
    let flags = CM_GETIDLIST_FILTER_ENUMERATOR | CM_GETIDLIST_FILTER_PRESENT;

    // null termination
    let mut filter_bytes = filter.as_bytes().to_vec();
    filter_bytes.push(0);

    // https://learn.microsoft.com/en-us/windows/win32/api/cfgmgr32/nf-cfgmgr32-cm_get_device_id_list_sizea
    let ret = unsafe {
        CM_Get_Device_ID_List_SizeA(&mut size, PCSTR::from_raw(filter_bytes.as_ptr()), flags)
    };
    if ret != CR_SUCCESS {
        let err = unsafe { CM_MapCrToWin32Err(ret, ERROR_INVALID_DATA.0) };
        return Err(Error::from(WIN32_ERROR(err)));
    }

    let mut buffer = vec![0u8; size as usize];

    // https://learn.microsoft.com/en-us/windows/win32/api/cfgmgr32/nf-cfgmgr32-cm_get_device_id_lista
    let ret = unsafe {
        CM_Get_Device_ID_ListA(
            PCSTR::from_raw(filter_bytes.as_ptr()),
            buffer.as_mut_slice(),
            flags,
        )
    };
    if ret != CR_SUCCESS {
        let err = unsafe { CM_MapCrToWin32Err(ret, ERROR_INVALID_DATA.0) };
        return Err(Error::from(WIN32_ERROR(err)));
    }

    Ok(multi_sz(&buffer))
}

fn get_device_inst(id: &str) -> Result<u32, Error> {
    // https://learn.microsoft.com/en-us/windows/win32/api/cfgmgr32/nf-cfgmgr32-cm_locate_devnodea
    let mut devinst: u32 = 0;

    // null termination
    let mut buffer = id.as_bytes().to_vec();
    buffer.push(0);

    let ret = unsafe {
        CM_Locate_DevNodeA(
            &mut devinst,
            PCSTR::from_raw(buffer.as_ptr()),
            CM_LOCATE_DEVNODE_NORMAL,
        )
    };
    if ret != CR_SUCCESS {
        let err = unsafe { CM_MapCrToWin32Err(ret, ERROR_INVALID_DATA.0) };
        return Err(Error::from(WIN32_ERROR(err)));
    }

    Ok(devinst)
}

fn get_device_prop(devinst: u32, key: &PROPERTYKEY) -> Result<(Bytes, DEVPROPTYPE), Error> {
    // https://learn.microsoft.com/en-us/windows/win32/api/cfgmgr32/nf-cfgmgr32-cm_get_devnode_propertyw
    let pkey = DEVPROPKEY {
        fmtid: key.fmtid,
        pid: key.pid,
    };
    let mut prop_ty = DEVPROPTYPE::default();
    let mut size: u32 = 0;

    let ret = unsafe { CM_Get_DevNode_PropertyW(devinst, &pkey, &mut prop_ty, None, &mut size, 0) };
    if ret != CR_BUFFER_SMALL {
        let err = unsafe { CM_MapCrToWin32Err(ret, ERROR_INVALID_DATA.0) };
        return Err(Error::from(WIN32_ERROR(err)));
    }

    let mut buffer = vec![0u8; size as usize];

    let ret = unsafe {
        CM_Get_DevNode_PropertyW(
            devinst,
            &pkey,
            &mut prop_ty,
            Some(buffer.as_mut_ptr()),
            &mut size,
            0,
        )
    };
    if ret != CR_SUCCESS {
        let err = unsafe { CM_MapCrToWin32Err(ret, ERROR_INVALID_DATA.0) };
        return Err(Error::from(WIN32_ERROR(err)));
    }

    Ok((Bytes::from(buffer), prop_ty))
}

fn wide_to_multi(value: &mut Bytes) -> Result<Bytes, Error> {
    // https://learn.microsoft.com/en-us/windows/win32/api/stringapiset/nf-stringapiset-widechartomultibyte
    let mut wide = vec![];
    while value.remaining() != 0 {
        wide.push(value.get_u16_le());
    }

    let size =
        unsafe { WideCharToMultiByte(CP_ACP, WC_COMPOSITECHECK, &wide, None, PCSTR::null(), None) };
    if size == 0 {
        return Err(Error::from_thread());
    }

    let mut buffer = vec![0u8; size as usize];

    let size = unsafe {
        WideCharToMultiByte(
            CP_ACP,
            WC_COMPOSITECHECK,
            &wide,
            Some(buffer.as_mut_slice()),
            PCSTR::null(),
            None,
        )
    };
    if size == 0 {
        return Err(Error::from_thread());
    }

    Ok(Bytes::from(buffer))
}
