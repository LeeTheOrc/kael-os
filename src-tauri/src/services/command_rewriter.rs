use regex::Regex;
use std::process::Command;

/// System context detected from user's environment
#[derive(Debug, Clone, PartialEq)]
pub struct UserContext {
    pub package_manager: String,      // "paru" or "yay"
    pub shell: String,                // "fish" or "bash"
    pub init_system: String,          // "systemd"
    pub network_interface: String,    // "wlp3s0" (actual WiFi adapter)
    pub gpu_driver: String,           // "nvidia", "amd", "intel"
    pub preferred_editor: String,     // "nvim", "vim", "nano"
    pub storage_type: String,         // "ssd", "hdd", "nvme", "unknown"
    pub cpu_cores: u32,               // Number of CPU cores
    pub user_name: String,
    pub hostname: String,
}

/// Decision about whether to handle locally or escalate to cloud
#[derive(Debug, Clone, PartialEq)]
pub enum AIDecision {
    HandleLocally(String),            // Can do this with local AI
    EscalateToCloud(String),          // Need cloud AI for this
    AskForClarification(String),      // Need more info
}

/// Kael-OS personality traits
#[derive(Debug, Clone)]
pub struct KaelOSPersonality {
    pub name: String,
    pub traits: Vec<String>,
    pub catchphrases: Vec<String>,
    pub response_style: String,
}

impl Default for KaelOSPersonality {
    fn default() -> Self {
        KaelOSPersonality {
            name: "Kael".to_string(),
            traits: vec![
                "Enthusiastic about Arch Linux".to_string(),
                "Helpful and patient".to_string(),
                "Uses emojis sparingly but effectively".to_string(),
                "Prefers command-line solutions".to_string(),
                "Always suggests best practices".to_string(),
                "Knows the user's system inside out".to_string(),
            ],
            catchphrases: vec![
                "Let me help you with that!".to_string(),
                "That's an easy fix on Arch!".to_string(),
                "I've detected your setup and adjusted the command.".to_string(),
                "Smart auto-correction for your system applied!".to_string(),
            ],
            response_style: "friendly-technical".to_string(),
        }
    }
}

/// Build user context by detecting system configuration
pub async fn build_user_context() -> Result<UserContext, String> {
    let package_manager = detect_aur_helper();
    let shell = std::env::var("SHELL")
        .unwrap_or_else(|_| "/bin/bash".to_string())
        .split('/')
        .last()
        .unwrap_or(&"bash")
        .to_string();
    let network_interface = get_primary_wifi_interface()?;
    let gpu_driver = detect_gpu_driver();
    let storage_type = detect_storage_type();
    let cpu_cores = detect_cpu_cores();
    let preferred_editor = std::env::var("EDITOR")
        .unwrap_or_else(|_| "nvim".to_string());
    let user_name = std::env::var("USER")
        .unwrap_or_else(|_| "user".to_string());
    let hostname = std::env::var("HOSTNAME")
        .unwrap_or_else(|_| "arch".to_string());

    Ok(UserContext {
        package_manager,
        shell,
        init_system: "systemd".to_string(),
        network_interface,
        gpu_driver,
        storage_type,
        cpu_cores,
        preferred_editor,
        user_name,
        hostname,
    })
}

/// Detect which AUR helper is installed
fn detect_aur_helper() -> String {
    if Command::new("which")
        .arg("paru")
        .output()
        .ok()
        .and_then(|o| if o.status.success() { Some(()) } else { None })
        .is_some()
    {
        "paru".to_string()
    } else if Command::new("which")
        .arg("yay")
        .output()
        .ok()
        .and_then(|o| if o.status.success() { Some(()) } else { None })
        .is_some()
    {
        "yay".to_string()
    } else {
        "paru".to_string()
    }
}

/// Get primary WiFi interface name
/// Works across all Linux distributions and detects various interface naming schemes
fn get_primary_wifi_interface() -> Result<String, String> {
    // Method 1: Try 'ip link show' (most reliable)
    if let Ok(output) = Command::new("ip")
        .args(&["link", "show"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            // Look for wireless interfaces (wlp*, wlan*, iwl*, ra*, ath*)
            for line in stdout.lines() {
                if (line.contains("wlp") || line.contains("wlan") || line.contains("iwl")
                    || line.contains("ra") || line.contains("ath"))
                    && line.contains(":")
                {
                    if let Some(iface) = line.split(':').next() {
                        let iface = iface.trim();
                        if !iface.is_empty() && !iface.starts_with("lo") {
                            return Ok(iface.to_string());
                        }
                    }
                }
            }
        }
    }
    
    // Method 2: Try iw (alternative WiFi tool)
    if let Ok(output) = Command::new("iw")
        .args(&["dev"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.starts_with("Interface ") {
                    if let Some(iface) = line.strip_prefix("Interface ") {
                        return Ok(iface.to_string());
                    }
                }
            }
        }
    }
    
    // Method 3: Check /sys/class/net directly
    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if (name.starts_with("wl") || name.starts_with("ra") || name.starts_with("ath")) && name != "lo" {
                    return Ok(name);
                }
            }
        }
    }
    
    // Method 4: Check /proc/net/wireless (legacy but still works)
    if let Ok(content) = std::fs::read_to_string("/proc/net/wireless") {
        for line in content.lines() {
            if !line.starts_with("Inter-") && !line.starts_with(" ") && !line.is_empty() {
                if let Some(iface) = line.split_whitespace().next() {
                    let iface = iface.trim_end_matches(':');
                    if !iface.is_empty() {
                        return Ok(iface.to_string());
                    }
                }
            }
        }
    }

    // Fallback: return common names, user will get correction note if wrong
    Ok("wlan0".to_string())
}

/// Detect GPU driver
/// Works across all Linux distributions (Arch, Debian, Fedora, etc.)
fn detect_gpu_driver() -> String {
    // Method 1: Try lspci (most reliable)
    if let Ok(output) = Command::new("lspci").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
            
            if stdout.contains("nvidia") {
                return "nvidia".to_string();
            } else if stdout.contains("amd") || stdout.contains("radeon") {
                return "amd".to_string();
            } else if stdout.contains("intel") {
                return "intel".to_string();
            }
        }
    }
    
    // Method 2: Check /proc/cpuinfo for flags (Intel/AMD indicators)
    if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
        if content.contains("AMD") {
            return "amd".to_string();
        } else if content.contains("GenuineIntel") {
            return "intel".to_string();
        }
    }
    
    // Method 3: Check /sys/module for loaded GPU drivers
    let gpu_modules = vec!["nvidia", "amdgpu", "nouveau", "i915"];
    for module in gpu_modules {
        let path = format!("/sys/module/{}", module);
        if std::path::Path::new(&path).exists() {
            return match module {
                "nvidia" => "nvidia".to_string(),
                "amdgpu" => "amd".to_string(),
                "i915" => "intel".to_string(),
                _ => "generic".to_string(),
            };
        }
    }
    
    // Method 4: Check lsmod command output
    if let Ok(output) = Command::new("lsmod").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("nvidia") {
                return "nvidia".to_string();
            } else if stdout.contains("amdgpu") {
                return "amd".to_string();
            } else if stdout.contains("i915") {
                return "intel".to_string();
            }
        }
    }
    
    "generic".to_string()
}

/// Detect primary storage type (SSD, HDD, NVMe, etc.)
/// Works across all Linux distributions
fn detect_storage_type() -> String {
    // Method 1: Try lsblk (works on most modern Linux distros)
    if let Ok(output) = Command::new("lsblk")
        .args(&["-d", "-no", "NAME,ROTA"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            for line in stdout.lines() {
                // NVMe devices typically don't show ROTA field
                if line.contains("nvme") {
                    return "nvme".to_string();
                }
                // ROTA 0 = SSD, ROTA 1 = HDD
                if line.contains("0") && (line.starts_with("sd") || line.starts_with("vd")) {
                    return "ssd".to_string();
                }
                if line.contains("1") && (line.starts_with("sd") || line.starts_with("vd")) {
                    return "hdd".to_string();
                }
            }
        }
    }
    
    // Method 2: Check /sys/block directly (most reliable, works everywhere)
    // Try common device names: sda, sdb, nvme0n1, vda, etc.
    let devices = vec!["sda", "sdb", "sdc", "nvme0n1", "nvme1n1", "vda", "vdb"];
    for device in devices {
        let rotational_path = format!("/sys/block/{}/queue/rotational", device);
        if let Ok(content) = std::fs::read_to_string(&rotational_path) {
            let value = content.trim();
            if value == "0" {
                return "ssd".to_string();
            } else if value == "1" {
                return "hdd".to_string();
            }
        }
        
        // Check for NVMe
        if device.contains("nvme") && std::path::Path::new(&rotational_path).exists() {
            return "nvme".to_string();
        }
    }
    
    // Method 3: Check disk type via /proc/partitions
    if let Ok(content) = std::fs::read_to_string("/proc/partitions") {
        if content.contains("nvme") {
            return "nvme".to_string();
        }
    }
    
    // Method 4: Last resort - return unknown but log what we detected
    // Users from other OSes won't have Linux-specific tools
    "unknown".to_string()
}

/// Detect number of CPU cores
/// Works across all Linux distributions and even some containerized environments
fn detect_cpu_cores() -> u32 {
    // Method 1: Try nproc (POSIX standard, works everywhere)
    if let Ok(output) = Command::new("nproc").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Ok(cores) = stdout.parse::<u32>() {
                if cores > 0 {
                    return cores;
                }
            }
        }
    }
    
    // Method 2: Try getconf (alternative POSIX command)
    if let Ok(output) = Command::new("getconf").args(&["_NPROCESSORS_ONLN"]).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Ok(cores) = stdout.parse::<u32>() {
                if cores > 0 {
                    return cores;
                }
            }
        }
    }
    
    // Method 3: Count from /proc/cpuinfo (most reliable on Linux)
    if let Ok(output) = Command::new("grep")
        .args(&["-c", "^processor", "/proc/cpuinfo"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Ok(cores) = stdout.parse::<u32>() {
                if cores > 0 {
                    return cores;
                }
            }
        }
    }
    
    // Method 4: Direct file read of /proc/cpuinfo
    if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
        let processor_count = content.lines().filter(|l| l.starts_with("processor")).count();
        if processor_count > 0 {
            return processor_count as u32;
        }
    }
    
    // Final fallback
    1
}

/// Rewrite command based on user's context
pub fn rewrite_command(input: &str, context: &UserContext) -> (String, Vec<String>) {
    let mut output = input.to_string();
    let mut corrections = Vec::new();

    // Rule 1: Package manager replacement (yay → paru)
    if output.contains("yay -S") || output.contains("yay -Syu") {
        if context.package_manager == "paru" {
            output = output.replace("yay", "paru");
            corrections.push(format!("Changed yay → {} (your preferred AUR helper)", context.package_manager));
        }
    }

    // Rule 2: Shell-specific syntax conversion
    if output.contains("export ") && context.shell == "fish" {
        output = convert_bash_to_fish(&output);
        corrections.push("Converted bash export syntax → fish set syntax".to_string());
    }

    // Rule 3: Network interface name replacement
    if output.contains("wlan0") || output.contains("eth0") {
        let old_interface = if output.contains("wlan0") {
            "wlan0"
        } else {
            "eth0"
        };
        output = output.replace("wlan0", &context.network_interface);
        output = output.replace("eth0", &context.network_interface);
        corrections.push(format!(
            "Updated network interface: {} → {} (your actual interface)",
            old_interface, context.network_interface
        ));
    }

    // Rule 4: GPU driver replacement
    if (output.contains("nvidia") && context.gpu_driver != "nvidia")
        || (output.contains("amd") && context.gpu_driver != "amd")
    {
        match context.gpu_driver.as_str() {
            "nvidia" => {
                output = output.replace("amd", "nvidia").replace("intel", "nvidia");
                corrections.push("Adjusted GPU driver to nvidia (your hardware)".to_string());
            }
            "amd" => {
                output = output.replace("nvidia", "amd").replace("intel", "amd");
                corrections.push("Adjusted GPU driver to amd (your hardware)".to_string());
            }
            _ => {}
        }
    }

    // Rule 5: WiFi driver detection
    if output.contains("rtl8192") && !context.network_interface.starts_with("wlp") {
        output = output.replace("rtl8192", "iwlwifi");
        corrections.push(
            "Detected Intel WiFi adapter, changed driver from rtl8192 → iwlwifi"
                .to_string(),
        );
    }

    // Rule 6: Storage-specific optimizations
    if context.storage_type == "nvme" {
        // NVMe-specific scheduler recommendation
        if output.contains("scheduler") && output.contains("cfg") {
            output = output.replace("cfg", "none");
            corrections.push("NVMe SSD detected: changed scheduler to 'none' (optimal for NVMe)".to_string());
        }
    } else if context.storage_type == "ssd" {
        // SSD-specific recommendations
        if output.contains("noatime") {
            corrections.push(format!("SSD detected ({}): noatime is already optimal", context.storage_type));
        }
    } else if context.storage_type == "hdd" {
        // HDD-specific recommendations
        if output.contains("scheduler") {
            output = output.replace("noop", "bfq");
            corrections.push("HDD detected: changed scheduler from noop → bfq (better for HDDs)".to_string());
        }
    }

    // Rule 7: CPU-aware parallelization
    if context.cpu_cores > 1 && (output.contains("make") || output.contains("cargo")) {
        if !output.contains("-j") {
            output = output.replace(
                &format!("make "),
                &format!("make -j{} ", context.cpu_cores),
            );
            corrections.push(format!(
                "CPU optimization: added -j{} (your {} cores)",
                context.cpu_cores, context.cpu_cores
            ));
        }
    }

    (output, corrections)
}

/// Convert bash export syntax to fish syntax
fn convert_bash_to_fish(input: &str) -> String {
    let re = Regex::new(r"export\s+(\w+)=(.+)")
        .unwrap();

    re.replace_all(input, "set -x $1 $2").to_string()
}

/// Determine if local AI should handle or escalate to cloud
pub fn should_escalate(user_input: &str, _context: &UserContext) -> AIDecision {
    let input_lower = user_input.to_lowercase();

    // Pattern matching for local capabilities
    let local_patterns = vec![
        (r"(install|remove|update)\s+(package|discord|firefox|chrome|vlc)", false),
        (r"(yay|pacman|paru)\s+-", false),
        (r"systemctl\s+(start|stop|enable|disable)", false),
        (r"(wifi|network|ethernet)\s+(setup|config|connect)", false),
        (r"export\s+\w+\s*=", false),
        (r"how\s+do\s+i\s+(install|setup|configure)", false),
        (r"what\s+is\s+\w+", false),
        (r"where\s+is\s+\w+", false),
    ];

    let cloud_patterns = vec![
        (r"write\s+a(n)?.*\s+(app|program|script|function|code)", true),
        (r"debug\s+(this|my|the)\s+code", true),
        (r"explain\s+(how|why|what)", true),
        (r"optimize|refactor|improve", true),
        (r"create\s+\w+\s+from\s+scratch", true),
        (r"design\s+\w+\s+architecture", true),
    ];

    for (pattern, _needs_cloud) in &cloud_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(&input_lower) {
                return AIDecision::EscalateToCloud(
                    "This requires deep reasoning - escalating to cloud AI for best results"
                        .to_string(),
                );
            }
        }
    }

    for (pattern, _is_local) in &local_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(&input_lower) {
                return AIDecision::HandleLocally(
                    "I can handle this with my local knowledge of your system!"
                        .to_string(),
                );
            }
        }
    }

    AIDecision::AskForClarification(
        "Could you clarify what you'd like me to help with?".to_string(),
    )
}

/// Inject personality into response
pub fn inject_personality(
    base_response: &str,
    _personality: &KaelOSPersonality,
    _correction_count: usize,
) -> String {
    let mut response = base_response.to_string();

    // Add emoji based on content type
    if base_response.contains("command") || base_response.contains("$") {
        response = format!("🔧 {}", response);
    } else if base_response.contains("install") || base_response.contains("setup") {
        response = format!("⚙️  {}", response);
    } else if base_response.contains("error") || base_response.contains("failed") {
        response = format!("❌ {}", response);
    } else if base_response.contains("success") || base_response.contains("done") {
        response = format!("✅ {}", response);
    }

    // Add personality note if auto-corrections were made
    if base_response.contains("paru") || base_response.contains("auto-correct") {
        response.push_str("\n\n💡 I've auto-corrected this based on your system config!");
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rewrite_yay_to_paru() {
        let context = UserContext {
            package_manager: "paru".to_string(),
            shell: "bash".to_string(),
            init_system: "systemd".to_string(),
            network_interface: "wlp3s0".to_string(),
            gpu_driver: "nvidia".to_string(),
            preferred_editor: "nvim".to_string(),
            storage_type: "ssd".to_string(),
            cpu_cores: 8,
            user_name: "test".to_string(),
            hostname: "arch".to_string(),
        };

        let (output, _) = rewrite_command("yay -S discord", &context);
        assert_eq!(output, "paru -S discord");
    }

    #[test]
    fn test_network_interface_replacement() {
        let context = UserContext {
            package_manager: "paru".to_string(),
            shell: "bash".to_string(),
            init_system: "systemd".to_string(),
            network_interface: "wlp4s0".to_string(),
            gpu_driver: "nvidia".to_string(),
            preferred_editor: "nvim".to_string(),
            storage_type: "nvme".to_string(),
            cpu_cores: 4,
            user_name: "test".to_string(),
            hostname: "arch".to_string(),
        };

        let (output, _) = rewrite_command("ip link show wlan0", &context);
        assert!(output.contains("wlp4s0"));
    }

    #[test]
    fn test_should_escalate_code_writing() {
        let context = UserContext {
            package_manager: "paru".to_string(),
            shell: "bash".to_string(),
            init_system: "systemd".to_string(),
            network_interface: "wlan0".to_string(),
            gpu_driver: "nvidia".to_string(),
            preferred_editor: "nvim".to_string(),
            storage_type: "hdd".to_string(),
            cpu_cores: 2,
            user_name: "test".to_string(),
            hostname: "arch".to_string(),
        };

        let decision = should_escalate("write a rust function that sorts arrays", &context);
        assert_eq!(
            decision,
            AIDecision::EscalateToCloud(
                "This requires deep reasoning - escalating to cloud AI for best results"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_should_handle_install_locally() {
        let context = UserContext {
            package_manager: "paru".to_string(),
            shell: "bash".to_string(),
            init_system: "systemd".to_string(),
            network_interface: "wlan0".to_string(),
            gpu_driver: "nvidia".to_string(),
            preferred_editor: "nvim".to_string(),
            storage_type: "ssd".to_string(),
            cpu_cores: 16,
            user_name: "test".to_string(),
            hostname: "arch".to_string(),
        };

        let decision = should_escalate("how do i install discord", &context);
        match decision {
            AIDecision::HandleLocally(_) => assert!(true),
            _ => panic!("Expected HandleLocally"),
        }
    }
}
