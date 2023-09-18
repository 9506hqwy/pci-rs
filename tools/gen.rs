use quote::quote;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args().nth(1).ok_or("Not specify file path")?;
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut vendors = vec![];
    let mut vendor = Vendor::default();

    let mut classes = vec![];
    let mut cls = BaseClass::default();

    let mut class_sec = false;

    for line in reader.lines().flatten() {
        if line.len() < 4 {
            continue;
        }

        if line.starts_with('#') {
            continue;
        }

        if !class_sec {
            if let Some((id, name)) = parse4_name(&line) {
                if vendor.id != 0 {
                    vendors.push(vendor);
                }

                vendor = Vendor {
                    id,
                    name,
                    devices: vec![],
                };
            } else if let Some(result) = line.strip_prefix("\t\t") {
                if let Some((sub_vendor, name)) = parse4_name(result) {
                    if let Some((sub_device, name)) = parse4_name(&name) {
                        let subsystem = SubSystem {
                            sub_vendor,
                            sub_device,
                            name,
                        };

                        vendor
                            .devices
                            .last_mut()
                            .unwrap()
                            .subsystems
                            .push(subsystem);
                    }
                }
            } else if let Some(result) = line.strip_prefix('\t') {
                if let Some((id, name)) = parse4_name(result) {
                    let device = Device {
                        id,
                        name,
                        subsystems: vec![],
                    };

                    vendor.devices.push(device);
                }
            } else if let Some(result) = line.strip_prefix("C ") {
                class_sec = true;
                if let Some((id, name)) = parse2_name(result) {
                    cls = BaseClass {
                        id,
                        name,
                        sub_classes: vec![],
                    };
                }
            }
        } else if let Some(result) = line.strip_prefix("C ") {
            if let Some((id, name)) = parse2_name(result) {
                if cls.id != 0 {
                    classes.push(cls);
                }

                cls = BaseClass {
                    id,
                    name,
                    sub_classes: vec![],
                };
            }
        } else if let Some(result) = line.strip_prefix("\t\t") {
            if let Some((id, name)) = parse2_name(result) {
                let prog_if = ProgIf { id, name };

                cls.sub_classes.last_mut().unwrap().prog_ifs.push(prog_if);
            }
        } else if let Some(result) = line.strip_prefix('\t') {
            if let Some((id, name)) = parse2_name(result) {
                let sub_class = SubClass {
                    id,
                    name,
                    prog_ifs: vec![],
                };

                cls.sub_classes.push(sub_class);
            }
        }
    }

    let ids = gen(vendors, classes)?;

    println!("{}", ids);
    Ok(())
}

fn parse2_name(line: &str) -> Option<(u8, String)> {
    match u8::from_str_radix(&line[0..2], 16) {
        Ok(id) => Some((id, line[2..].trim().to_string())),
        _ => None,
    }
}

fn parse4_name(line: &str) -> Option<(u16, String)> {
    match u16::from_str_radix(&line[0..4], 16) {
        Ok(id) => Some((id, line[4..].trim().to_string())),
        _ => None,
    }
}

fn gen(vendors: Vec<Vendor>, classes: Vec<BaseClass>) -> Result<String, Box<dyn Error>> {
    let mut vmodels = vec![];
    for vendor in &vendors {
        let mut devices = vec![];
        for device in &vendor.devices {
            let id = device.id;
            let name = &device.name;
            devices.push(quote! {
                d.push(Device{ id: #id, name: #name, subsystems: vec![] });
            });
        }

        let id = vendor.id;
        let name = &vendor.name;
        if !vendor.devices.is_empty() {
            vmodels.push(quote! {
                let mut d = vec![];
                #(#devices)*
                v.push(Vendor{ id: #id, name: #name, devices: d });
            });
        } else {
            vmodels.push(quote! {
                v.push(Vendor{ id: #id, name: #name, devices: vec![] });
            });
        }
    }

    let mut cmodels = vec![];
    for cls in &classes {
        let mut sub_classes = vec![];
        for sub_class in &cls.sub_classes {
            let id = sub_class.id;
            let name = &sub_class.name;
            sub_classes.push(quote! {
                s.push(SubClass{ id: #id, name: #name, prog_ifs: vec![] });
            });
        }

        let id = cls.id;
        let name = &cls.name;
        if !cls.sub_classes.is_empty() {
            cmodels.push(quote! {
                let mut s = vec![];
                #(#sub_classes)*
                c.push(BaseClass{ id: #id, name: #name, sub_classes: s });
            });
        } else {
            cmodels.push(quote! {
                c.push(BaseClass{ id: #id, name: #name, sub_classes: vec![] });
            });
        }
    }

    let ids = quote! {
        #![allow(clippy::vec_init_then_push)]

        use once_cell::sync::Lazy;

        static VENDORS: Lazy<Vec<Vendor>> = Lazy::new(|| {
            let mut v = vec![];
            #(#vmodels)*
            v
        });

        static CLASSES: Lazy<Vec<BaseClass>> = Lazy::new(|| {
            let mut c = vec![];
            #(#cmodels)*
            c
        });

        pub fn get_vendor(id: u16) -> Option<&'static Vendor> {
            VENDORS.iter().find(|v| v.id == id)
        }

        pub fn get_class(id: u8) -> Option<&'static BaseClass> {
            CLASSES.iter().find(|c| c.id == id)
        }

        pub struct Vendor {
            id: u16,
            name: &'static str,
            devices: Vec<Device>,
        }

        impl Vendor {
            pub fn id(&self) -> u16 {
                self.id
            }

            pub fn name(&self) -> &'static str {
                self.name
            }

            pub fn get_device(&self, id: u16) -> Option<&Device> {
                self.devices.iter().find(|d| d.id == id)
            }
        }

        pub struct Device {
            id: u16,
            name: &'static str,
            subsystems: Vec<SubSystem>,
        }

        impl Device {
            pub fn id(&self) -> u16 {
                self.id
            }

            pub fn name(&self) -> &'static str {
                self.name
            }

            pub fn get_subsystem(&self, vendor: u16, device: u16) -> Option<&SubSystem> {
                self.subsystems.iter().find(|s| s.sub_vendor == vendor && s.sub_device == device)
            }
        }

        pub struct SubSystem {
            sub_vendor: u16,
            sub_device: u16,
            name: &'static str,
        }

        impl SubSystem {
            pub fn sub_vendor(&self) -> u16 {
                self.sub_vendor
            }

            pub fn sub_device(&self) -> u16 {
                self.sub_device
            }

            pub fn name(&self) -> &'static str {
                self.name
            }
        }

        pub struct BaseClass {
            id: u8,
            name:  &'static str,
            sub_classes: Vec<SubClass>,
        }

        impl BaseClass {
            pub fn id(&self) -> u8 {
                self.id
            }

            pub fn name(&self) -> &'static str {
                self.name
            }

            pub fn get_sub_class(&self, id: u8) -> Option<&SubClass> {
                self.sub_classes.iter().find(|c| c.id == id)
            }
        }

        pub struct SubClass {
            id: u8,
            name:  &'static str,
            prog_ifs: Vec<ProgIf>,
        }

        impl SubClass {
            pub fn id(&self) -> u8 {
                self.id
            }

            pub fn name(&self) -> &'static str {
                self.name
            }

            pub fn get_prog_if(&self, id: u8) -> Option<&ProgIf> {
                self.prog_ifs.iter().find(|c| c.id == id)
            }
        }

        pub struct ProgIf {
            id: u8,
            name: &'static str,
        }

        impl ProgIf {
            pub fn id(&self) -> u8 {
                self.id
            }

            pub fn name(&self) -> &'static str {
                self.name
            }
        }
    };

    Ok(ids.to_string())
}

#[derive(Default)]
struct Vendor {
    id: u16,
    name: String,
    devices: Vec<Device>,
}

#[derive(Default)]
struct Device {
    id: u16,
    name: String,
    subsystems: Vec<SubSystem>,
}

#[derive(Default)]
struct SubSystem {
    sub_vendor: u16,
    sub_device: u16,
    name: String,
}

#[derive(Default)]
struct BaseClass {
    id: u8,
    name: String,
    sub_classes: Vec<SubClass>,
}

#[derive(Default)]
struct SubClass {
    id: u8,
    name: String,
    prog_ifs: Vec<ProgIf>,
}

#[derive(Default)]
struct ProgIf {
    id: u8,
    name: String,
}
