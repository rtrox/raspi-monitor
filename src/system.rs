use std::net::{IpAddr, Ipv4Addr};

use sysinfo::{Components, Disks, MemoryRefreshKind, Networks, System};

static RASPI_CPU_TEMP: &str = "cpu_thermal temp1";
// pub const RASPI_ADC_TEMP: &str = "rp1_adc temp1";

static ROOT_MOUNT_POINT: &str = "/";

pub struct SysInfo {
    sys: sysinfo::System,
    components: sysinfo::Components,
    networks: sysinfo::Networks,
    disks: sysinfo::Disks,
}

impl SysInfo {
    pub fn new() -> Self {
        let sys = System::new_all();
        let components = Components::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();
        Self {
            sys: sys,
            components: components,
            networks: networks,
            disks: disks,
        }
    }

    pub fn refresh(&mut self) {
        self.sys.refresh_cpu_usage();
        self.sys
            .refresh_memory_specifics(MemoryRefreshKind::new().with_ram());
        self.components.refresh();
        self.networks.refresh();
        self.disks.refresh()
    }

    pub fn hostname() -> String {
        System::host_name().unwrap_or("".to_string())
    }

    pub fn ip_addr(&self) -> IpAddr {
        for (iface_name, net) in self.networks.iter() {
            if iface_name == "eth0" || iface_name == "wlan0" {
                for ip in net.ip_networks() {
                    if ip.addr.is_ipv4() {
                        return ip.addr;
                    }
                }
            }
        }
        return IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
    }

    pub fn uptime() -> String {
        let up = System::uptime();
        let seconds = up % 60;
        let minutes = (up / 60) % 60;
        let hours = up / 3600 % 24;
        let days: u64 = hours / 24;
        if days > 0 {
            format!("{}d{}h{}m", days, hours, minutes)
        } else {
            format!("{}h{}m{}s", hours, minutes, seconds)
        }
    }

    pub fn cpu_usage(&self) -> f32 {
        let mut sum = 0.0;
        for cpu in self.sys.cpus() {
            sum += cpu.cpu_usage();
        }
        sum / self.sys.cpus().len() as f32
    }

    pub fn memory_usage(&self) -> f32 {
        self.sys.used_memory() as f32 / self.sys.total_memory() as f32 * 100.0
    }

    pub fn cpu_temp(&self) -> f32 {
        for component in &self.components {
            if component.label() == RASPI_CPU_TEMP {
                return component.temperature();
            }
        }
        0.0
    }

    pub fn root_disk_usage(&self) -> Option<f32> {
        for disk in self.disks.iter() {
            if disk.mount_point().to_str().unwrap_or("") == ROOT_MOUNT_POINT {
                return Some(
                    (disk.total_space() - disk.available_space()) as f32
                        / disk.total_space() as f32
                        * 100.0,
                );
            }
        }
        None
    }
}
