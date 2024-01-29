use pci::{ids, PciConfig};
use std::env;

#[derive(Default)]
struct Option {
    n: bool,
    nn: bool,
    v: bool,
}

fn main() {
    let mut option = Option::default();
    for arg in env::args() {
        match arg.as_str() {
            "-n" => {
                option.n = true;
            }
            "-nn" => {
                option.nn = true;
            }
            "-v" => {
                option.v = true;
            }
            _ => {}
        }
    }

    unsafe { libc::iopl(3) };

    let mut devices = scan_device(0);
    devices.sort_by_key(|d| d.0);

    for (bus, device, func, v) in devices {
        print_device(bus, device, func, &v, &option);
    }
}

fn scan_device(bus: u8) -> Vec<(u8, u8, u8, PciConfig)> {
    let mut devs = vec![];
    let mut sub_buses = vec![];

    for device in 0..32 {
        let v = pci::get_pci_config(bus, device, 0);
        if let Some(v) = &v {
            devs.push((bus, device, 0, v.clone()));

            if let Some(t1) = v.get_type1_header() {
                sub_buses.push(t1.secondary_bus_number());
            }

            if v.header_type().multi_function_device() {
                for func in 1..8 {
                    let v = pci::get_pci_config(bus, device, func);
                    if let Some(v) = &v {
                        devs.push((bus, device, func, v.clone()));

                        if let Some(t1) = v.get_type1_header() {
                            sub_buses.push(t1.secondary_bus_number());
                        }
                    }
                }
            }
        }
    }

    for sub_bus in sub_buses {
        let mut v = scan_device(sub_bus);
        devs.append(&mut v);
    }

    devs
}

fn print_device(bus: u8, device: u8, func: u8, cfg: &PciConfig, option: &Option) {
    print!("{:02x}:{:02x}.{} ", bus, device, func);

    let ccode = cfg.class_code();
    let base_class = ids::get_class(ccode.base_class()).unwrap();
    let sub_class = base_class.get_sub_class(ccode.sub_class()).unwrap();
    if option.n {
        print!("{:02x}{:02x}: ", base_class.id(), sub_class.id());
    } else {
        print!("{}", sub_class.name());
        if option.nn {
            print!(" [{:02x}{:02x}]", base_class.id(), sub_class.id());
        }

        print!(": ")
    }

    let vendor = ids::get_vendor(cfg.vendor_id()).unwrap();
    let device = vendor.get_device(cfg.device_id()).unwrap();
    if option.n {
        print!("{:04x}:{:04x}", vendor.id(), device.id());
    } else {
        print!("{} {}", vendor.name(), device.name());
        if option.nn {
            print!(" [{:04x}:{:04x}]", vendor.id(), device.id());
        }
    }

    if cfg.revision_id() == 0 {
        println!();
    } else {
        println!(" (rev {:02x})", cfg.revision_id());
    }

    if !option.v {
        return;
    }

    let cmd = cfg.command();
    println!("        Control: I/O{} Mem{} BusMaster{} SpecCycle{} MemWINV{} VGASnoop{} ParErr{} Stepping{} SERR{} FastB2B{} DisINTx{}",
             flag(cmd.io_space_enable()),
             flag(cmd.memory_space_enable()),
             flag(cmd.bus_master_enable()),
             flag(cmd.special_cycle_enable()),
             flag(cmd.memory_write_and_invalidate()),
             flag(cmd.vga_palette_snoop()),
             flag(cmd.parity_error_response()),
             flag(cmd.idsel_stepping_wait_cycle_control()),
             flag(cmd.serr_enable()),
             flag(cmd.fast_back_to_back_transactions_enable()),
             flag(cmd.interrupt_disable()),
    );

    let status = cfg.status();
    println!("        Status: Cap{} 66MHz{} UDF{} FastB2B{} ParErr{} DEVSEL={} >TAbort{} <TAbort{} <MAbort{} >SERR{} <PERR{} INTx{}",
             flag(status.capabilities_list()),
             flag(status.mhz_66_capable()),
             flag(status.user_definable_features()),
             flag(status.fast_back_to_back_transactions_capable()),
             flag(status.master_data_parity_error()),
             match status.devsel_timing() {
                 0 => "fast",
                 1 => "medium",
                 2 => "slow",
                 _ => "",
             },
             flag(status.signaled_target_abort()),
             flag(status.received_target_abort()),
             flag(status.signaled_system_error()),
             flag(status.received_master_abort()),
             flag(status.detected_parity_error()),
             flag(status.interrupt_status()),
    );

    if cmd.bus_master_enable() {
        print!("        Latency: {}", cfg.master_latency_timer());

        let cache_line_size = cfg.cache_line_size();
        if cache_line_size != 0 {
            print!(", Cache Line Size: {} bytes", cache_line_size * 4);
        }

        println!();
    }

    if let Some(t0) = cfg.get_type0_header() {
        if t0.subsystem_vendor_id() != 0 {
            if let Some(subsystem) =
                device.get_subsystem(t0.subsystem_vendor_id(), t0.subsystem_id())
            {
                println!("        Subsystem: {} {}", vendor.name(), subsystem.name());
            } else {
                let subsystem = ids::get_vendor(t0.subsystem_vendor_id()).unwrap();
                println!(
                    "        Subsystem: {} Device {:04x}",
                    subsystem.name(),
                    t0.subsystem_id()
                );
            }
        }

        for bar in t0.bars() {
            if bar.bar() == 0 {
                continue;
            }

            if bar.io_space() {
                print!("        I/O ports at {:04x}", bar.bar());
            } else {
                print!("        Memory at {:08x} (", bar.bar());

                if bar.b64() {
                    print!("64-bit");
                } else if bar.b32() {
                    print!("32-bit");
                } else if bar.b16() {
                    print!("low-1M");
                } else {
                    print!("type 3");
                }

                print!(
                    ", {}prefetchable)",
                    if bar.prefetchable() { "" } else { "non-" }
                );
            }

            println!();
        }
    }

    let rom = if let Some(t0) = cfg.get_type0_header() {
        Some(t0.expansion_rom())
    } else {
        cfg.get_type1_header().map(|t1| t1.expansion_rom())
    };

    if let Some(rom) = rom {
        if rom != 0 {
            print!("        Expansion  ROM at ");

            if (rom & 0xFFFF_F800) != 0 {
                print!("<ignored>");
            } else {
                print!("<unassigned>");
            }

            if (rom & 0x01) == 0 {
                print!(" [disabled]");
            }

            println!();
        }
    }

    if cfg.status().capabilities_list() {
        let mut cap_next = cfg.capabilities_pointer();
        let mut capability = cfg.capability();
        while capability.is_some() {
            let cap = capability.unwrap();
            println!("        Capabilities: [{:02x}] {:?}", cap_next, cap.id());
            cap_next = cap.next_pointer();
            capability = cap.next(cfg);
        }
    }
}

fn flag(f: bool) -> &'static str {
    if f {
        "+"
    } else {
        "-"
    }
}
