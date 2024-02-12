use std::{process::Command, time::Duration};
use sysinfo::{
    Components, CpuRefreshKind, Disks, MemoryRefreshKind, Networks, RefreshKind, System
};

fn clear_terminal() {
    if cfg!(windows) {
        let _ = Command::new("cmd").arg("/c").arg("cls").status();
    } else {
        let _ = Command::new("sh").arg("-c").arg("clear").status();
    }
}

async fn emit_beep(count: u64) {
    println!("Beep {}x", count);
    for i in 0..count {
        std::thread::sleep(Duration::from_millis((1000 / count) * i));
        println!("\x07");
    }
}

fn format_memory(value: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if value >= GB {
        format!("{:.2}GB", value as f64 / GB as f64)
    } else if value >= MB {
        format!("{:.2}MB", value as f64 / MB as f64)
    } else if value >= KB {
        format!("{:.2}KB", value as f64 / KB as f64)
    } else {
        format!("{} bytes", value)
    }
}

async fn fetch_and_display_system_info() {
    let max_temp = 70.0;
    let mut system = System::new_with_specifics(
        RefreshKind::new()
            .with_memory(MemoryRefreshKind::new().with_ram())
            .with_cpu(CpuRefreshKind::new().with_cpu_usage()),
    );
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    system.refresh_all();

    clear_terminal();

    let mut should_beep = false;
    let mut cpus_iter = system.cpus().iter();
    let mut components = Components::new_with_refreshed_list();
    let disks = Disks::new_with_refreshed_list();
    let networks = Networks::new_with_refreshed_list();
    let mut network_vec: Vec<_> = networks.iter().collect();
    network_vec.sort_by(|(name1, _), (name2, _)| name1.cmp(name2));
    components.sort_by_key(|component| {
        let label = component.label();
        let core_number = label
            .chars()
            .skip_while(|&c| !c.is_numeric())
            .collect::<String>()
            .parse::<usize>()
            .unwrap_or(0);
        core_number
    });

    // Filtrando somente os componentes que começam com "coretemp"
    let coretemps: Vec<_> = components
        .iter()
        .filter(|component| component.label().starts_with("coretemp"))
        .collect();
    let amdgpus: Vec<_> = components
        .iter()
        .filter(|component| component.label().contains("gpu"))
        .collect();

    println!("CPU: {}", system.cpus()[0].brand());
    for (index, core) in coretemps.iter().enumerate() {
        let temperature = core.temperature();
        if temperature > max_temp {
            should_beep = true;
            println!(
                "\x1b[31mCore {}: {:.0}% {:.1}ºC OVERHEAT\x1b[0m",
                index,
                cpus_iter.next().unwrap().cpu_usage(),
                temperature
            );
        } else {
            println!(
                "Core {}: {:.0}% {:.1}ºC",
                index,
                cpus_iter.next().unwrap().cpu_usage(),
                temperature
            );
        }
    }

    for (index, component) in amdgpus.iter().enumerate() {
        let temperature = component.temperature();
        if temperature > max_temp {
            should_beep = true;
            println!(
                "\x1b[31m{} {}: {:.1}ºC OVERHEAT\x1b[0m",
                component.label(),
                index,
                temperature
            );
        } else {
            println!("{} {}: {:.1}ºC", component.label(), index, temperature);
        }
    }

    println!(
        "RAM: Total:{} Used:{} Free:{} Avaliable:{}",
        format_memory(system.total_memory()),
        format_memory(system.used_memory()),
        format_memory(system.free_memory()),
        format_memory(system.available_memory())
    );
    println!(
        "SWAP: Total:{} Used:{} Free:{}",
        format_memory(system.total_swap()),
        format_memory(system.used_swap()),
        format_memory(system.free_swap()),
    );
    for disk in disks.list() {
        let used = disk.total_space() - disk.available_space();
        println!(
            "[{:?}] Total: {} Used:{} Free:{}",
            disk.name(),
            format_memory(disk.total_space()),
            format_memory(used),
            format_memory(disk.available_space())
        );
    }
    /*for (pid, process) in system.processes() {
        let disk_usage = process.disk_usage();
        println!(
            "[{}] read bytes   : new/total => {}/{} B",
            pid, disk_usage.read_bytes, disk_usage.total_read_bytes,
        );
        println!(
            "[{}] written bytes: new/total => {}/{} B",
            pid, disk_usage.written_bytes, disk_usage.total_written_bytes,
        );
    }*/
    for (name, network_data) in network_vec {
        println!(
            "{}: Received:{} Transmited:{} Total Received: {} Total Transmitted: {}",
            name,
            format_memory(network_data.received()),
            format_memory(network_data.transmitted()),
            format_memory(network_data.total_received()),
            format_memory(network_data.total_transmitted()),
        );
    }

    if should_beep {
        emit_beep(3).await;
    }
}

#[async_std::main]
async fn main() {
    fetch_and_display_system_info().await;

    loop {
        fetch_and_display_system_info().await;
        std::thread::sleep(Duration::from_secs(1));
    }
}
