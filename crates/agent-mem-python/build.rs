fn main() {
    // Use pyo3-build-config to find Python
    pyo3_build_config::use_pyo3_cfgs();
    
    // On macOS with Homebrew Python, we need to link against Python framework
    if cfg!(target_os = "macos") {
        // Get Python library path from python3-config
        if let Ok(output) = std::process::Command::new("python3-config")
            .args(&["--ldflags", "--embed"])
            .output()
        {
            let flags = String::from_utf8_lossy(&output.stdout);
            for flag in flags.split_whitespace() {
                if flag.starts_with("-L") {
                    println!("cargo:rustc-link-search=native={}", &flag[2..]);
                } else if flag.starts_with("-l") {
                    // Extract library name (e.g., -lpython3.14 -> python3.14)
                    let lib_name = &flag[2..];
                    println!("cargo:rustc-link-lib={}", lib_name);
                } else if flag == "-framework" {
                    // Framework linking is handled separately
                } else if flag.starts_with("-framework") {
                    let framework = &flag[11..];
                    println!("cargo:rustc-link-lib=framework={}", framework);
                }
            }
        } else if let Ok(output) = std::process::Command::new("python3-config")
            .args(&["--ldflags"])
            .output()
        {
            let flags = String::from_utf8_lossy(&output.stdout);
            for flag in flags.split_whitespace() {
                if flag.starts_with("-L") {
                    println!("cargo:rustc-link-search=native={}", &flag[2..]);
                } else if flag.starts_with("-l") && flag.contains("python") {
                    let lib_name = &flag[2..];
                    println!("cargo:rustc-link-lib={}", lib_name);
                }
            }
        }
    }
}

