use std::fs;
use std::process::Command;
use std::str::FromStr;
use std::time::Duration;

// Multi-colored NixOS ASCII logo (Gruvbox palette)
const NIXOS_LOGO: &[&str] = &[
    "\x1b[38;2;214;93;14m  ▗▄   \x1b[38;2;215;153;33m▗▄ ▄▖\x1b[0m",
    "\x1b[38;2;214;93;14m ▄▄🬸█▄▄▄\x1b[38;2;215;153;33m🬸█▛ \x1b[38;2;152;151;26m▃\x1b[0m",
    "\x1b[38;2;214;93;14m   ▟▛    \x1b[38;2;152;151;26m▜\x1b[38;2;69;133;136m▃▟🬕\x1b[0m",
    "\x1b[38;2;214;93;14m🬋🬋🬫█      \x1b[38;2;69;133;136m█🬛🬋🬋\x1b[0m",
    "\x1b[38;2;204;36;29m 🬷▛🮃\x1b[38;2;177;98;134m▙    \x1b[38;2;69;133;136m▟▛\x1b[0m",
    "\x1b[38;2;204;36;29m 🮃 \x1b[38;2;177;98;134m▟█🬴\x1b[38;2;204;36;29m▀▀▀█🬴▀▀\x1b[0m",
    "\x1b[38;2;177;98;134m  ▝▀ ▀▘   \x1b[38;2;204;36;29m▀▘\x1b[0m",
];

const DEFAULT_LOGO: &[&str] = &[
    "        /\\_/\\         ",
    "       ( o.o )        ",
    "        = ^ =         ",
    "                      ",
    "                      ",
    "                      ",
    "                      ",
    "                      ",
];

fn parse_u64(s: &str) -> u64 {
    if let Ok(val) = u64::from_str(s) {
        val
    } else {
        0
    }
}

fn parse_f64(s: &str) -> f64 {
    if let Ok(val) = f64::from_str(s) {
        val
    } else {
        0.0
    }
}

fn pad_key(s: &str) -> String {
    let mut out = s.to_string();
    let current = out.len();
    if current != 8 {
        let mut i = current;
        while i != 8 {
            out.push(' ');
            i += 1;
        }
    }
    out
}

fn command_output(command: &str, args: &[&str]) -> String {
    Command::new(command)
        .args(args)
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|_| "Unknown".to_string())
}

fn shell() -> String {
    std::env::var("SHELL")
        .unwrap_or_else(|_| "Unknown".into())
        .split('/')
        .last()
        .unwrap_or("Unknown")
        .to_string()
}

fn os() -> String {
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if let Some(name) = line.strip_prefix("PRETTY_NAME=") {
                return name.trim_matches('"').to_string();
            }
        }
    }
    "Unknown".into()
}

fn os_id() -> String {
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if let Some(id) = line.strip_prefix("ID=") {
                return id.trim_matches('"').to_lowercase();
            }
        }
    }
    "unknown".into()
}

fn kernel() -> String {
    command_output("uname", &["-r"])
}

fn cpu() -> String {
    if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
        for line in content.lines() {
            if line.starts_with("model name") {
                return line
                    .split(':')
                    .nth(1)
                    .unwrap_or("Unknown")
                    .trim()
                    .to_string();
            }
        }
    }
    "Unknown".into()
}

fn gpu() -> String {
    let smi_out = command_output("nvidia-smi", &["--query-gpu=gpu_name", "--format=csv,noheader,nounits"]);
    if smi_out != "Unknown" && !smi_out.is_empty() {
        return smi_out;
    }

    let lspci_out = command_output("lspci", &[]);
    if lspci_out != "Unknown" {
        for line in lspci_out.lines() {
            if (line.contains("VGA compatible controller") || line.contains("3D controller")) && line.contains("NVIDIA") {
                if let Some(idx) = line.find("NVIDIA Corporation ") {
                    let part = line.split_at(idx + 19).1;
                    let clean = part.replace("[", "").replace("]", "");
                    return clean.trim().to_string();
                }
                return line.to_string();
            }
        }
    }
    "Unknown".into()
}

fn memory() -> String {
    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        let mut total: u64 = 0;
        let mut available: u64 = 0;

        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                let val = line.split_whitespace().nth(1).unwrap_or("0");
                total = parse_u64(val);
            }

            if line.starts_with("MemAvailable:") {
                let val = line.split_whitespace().nth(1).unwrap_or("0");
                available = parse_u64(val);
            }
        }
        let used = total.saturating_sub(available);

        return format!(
            "{:.2} GB / {:.2} GB",
            used as f64 / 1024.0 / 1024.0,
            total as f64 / 1024.0 / 1024.0
        );
    }
    "Unknown".into()
}

fn uptime() -> String {
    if let Ok(content) = fs::read_to_string("/proc/uptime") {
        if let Some(seconds_str) = content.split_whitespace().next() {
            let seconds_f = parse_f64(seconds_str);
            if seconds_f != 0.0 {
                let secs = seconds_f as u64;
                let duration = Duration::from_secs(secs);
                let minutes = (duration.as_secs() % 3600) / 60;
                let hours = (duration.as_secs() % 86400) / 3600;
                let days = duration.as_secs() / 86400;

                if days != 0 {
                    return format!("{}d {}h {}m", days, hours, minutes);
                } else if hours != 0 {
                    return format!("{}h {}m", hours, minutes);
                } else {
                    return format!("{} mins", minutes);
                }
            }
        }
    }
    "Unknown".into()
}
fn os_age() -> String {
    if let Ok(metadata) = std::fs::metadata("/") {
        if let Ok(created) = metadata.created() {
            if let Ok(elapsed) = created.elapsed() {
                let days = elapsed.as_secs() / 86400;
                return format!("{} days", days);
            }
        }
    }
    "Unknown".into()
}

fn visual_width(s: &str) -> usize {
    let mut width = 0;
    let mut in_escape = false;

    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else {
            width += 1;
        }
    }
    width
}

fn main() {
    let distro = os_id();
    let logo_lines = match distro.as_str() {
        "nixos" => NIXOS_LOGO,
        _ => DEFAULT_LOGO,
    };

    let c_reset = "\x1b[0m";

    // Fastfetch exact key colors
    let c_distro = "\x1b[38;2;204;36;29m";  // Gruvbox Red (#cc241d) for OS/Kernel/Shell
    let c_system = "\x1b[38;2;69;133;136m"; // Gruvbox Cyan (#458588) for Uptime/CPU/GPU/RAM

    // Pac-Man color bar with Gruvbox hex sequence
    let pacman_bar = format!(
        "\x1b[38;2;215;153;33m󰮯 - \x1b[0m \x1b[38;2;204;36;29m\u{f02a0}\x1b[0m \x1b[38;2;152;151;26m\u{f02a0}\x1b[0m \x1b[38;2;69;133;136m\u{f02a0}\x1b[0m \x1b[38;2;177;98;134m\u{f02a0}\x1b[0m \x1b[38;2;214;93;14m\u{f02a0}\x1b[0m \x1b[38;2;235;219;178m\u{f02a0}\x1b[0m \x1b[38;2;146;131;116m\u{f02a0}\x1b[0m "
    );

    let info_lines = vec![
        format!("{}{}{} {}", c_distro, pad_key("os"), c_reset, os()),
        format!("{}{}{} Linux {}", c_distro, pad_key("ker"), c_reset, kernel()),
        format!("{}{}{} {}", c_distro, pad_key("sh"), c_reset, shell()),
        format!("{}{}{} {}", c_system, pad_key("up"), c_reset, uptime()),
        format!("{}{}{} {}", c_system, pad_key("cpu"), c_reset, cpu()),
        format!("{}{}{} {}", c_system, pad_key("gpu"), c_reset, gpu()),
        format!("{}{}{} {}", c_system, pad_key("ram"), c_reset, memory()),
        format!("{}{}{} {}", c_system, pad_key("os_age"), c_reset, os_age()),
        pacman_bar,
    ];

    let mut max_logo_width = 0;
    for line in logo_lines {
        let w = visual_width(line);
        max_logo_width = max_logo_width.max(w);
    }

    println!();
    let max_rows = logo_lines.len().max(info_lines.len());

    for i in 0..max_rows {
        let left_raw = logo_lines.get(i).copied().unwrap_or("");
        let current_w = visual_width(left_raw);
        let padding_amount = max_logo_width.saturating_sub(current_w);

        let mut padding = String::new();
        let mut p = 0;
        while p != padding_amount {
            padding.push(' ');
            p += 1;
        }

        let right = info_lines.get(i).map(|s| s.as_str()).unwrap_or("");

        println!("{}{}    {}", left_raw, padding, right);
    }
    println!();
}
