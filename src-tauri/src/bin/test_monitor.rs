use windows_capture::monitor::Monitor;

fn main() {
    let monitors = Monitor::enumerate().unwrap();
    for (i, m) in monitors.iter().enumerate() {
        println!("Monitor {}:", i);
        if let Ok(name) = m.name() {
            println!("  name: {}", name);
        }
    }
}
