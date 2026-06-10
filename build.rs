use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // ── Generate built-in theme registry ───────────────────────────────
    generate_builtin_themes(&manifest_dir, &out_dir);

    // ── PolicyKit installation (Linux only) ────────────────────────────
    if env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "linux" {
        return;
    }

    let policy_src = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("policy")
        .join("dev.toomosin.bludio.policy");

    if !policy_src.exists() {
        println!(
            "cargo:warning=PolicyKit policy file not found at {:?}",
            policy_src
        );
        return;
    }

    // Determine the target PolicyKit actions directory
    let policy_dir = env::var("BLUDIO_POLICY_DIR")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/share/polkit-1/actions"));

    let policy_dest = policy_dir.join("dev.toomosin.bludio.policy");

    // Check if the target directory is writable
    match fs::metadata(&policy_dir) {
        Ok(metadata) if metadata.is_dir() => {
            // Try a test write to check permissions
            let test_file = policy_dir.join(".bludio_write_test");
            match fs::write(&test_file, b"") {
                Ok(_) => {
                    let _ = fs::remove_file(&test_file);
                }
                Err(_) => {
                    println!(
                        "cargo:warning=Insufficient privileges to write to {:?}. PolicyKit policy was NOT installed.",
                        policy_dir
                    );
                    println!(
                        "cargo:warning=To enable privileged Bluetooth operations, run one of the following:"
                    );
                    println!(
                        "cargo:warning=  sudo cargo install --git https://github.com/toomosin/bludio"
                    );
                    println!(
                        "cargo:warning=  sudo cp policy/dev.toomosin.bludio.policy /usr/share/polkit-1/actions/"
                    );
                    return;
                }
            }
        }
        _ => {
            println!(
                "cargo:warning=PolicyKit actions directory {:?} does not exist. Policy was NOT installed.",
                policy_dir
            );
            println!(
                "cargo:warning=To enable privileged Bluetooth operations, manually copy the policy file:"
            );
            println!("cargo:warning=  sudo mkdir -p /usr/share/polkit-1/actions/");
            println!(
                "cargo:warning=  sudo cp policy/dev.toomosin.bludio.policy /usr/share/polkit-1/actions/"
            );
            return;
        }
    }

    // Copy the policy file
    match fs::copy(&policy_src, &policy_dest) {
        Ok(_) => {
            println!(
                "cargo:warning=PolicyKit policy installed to {:?}",
                policy_dest
            );
        }
        Err(e) => {
            println!(
                "cargo:warning=Failed to install PolicyKit policy to {:?}: {}",
                policy_dest, e
            );
            println!(
                "cargo:warning=To enable privileged Bluetooth operations, manually copy the policy file:"
            );
            println!(
                "cargo:warning=  sudo cp policy/dev.toomosin.bludio.policy /usr/share/polkit-1/actions/"
            );
        }
    }
}

/// Scan `assets/themes/light/` and `assets/themes/dark/` and generate a Rust
/// file that lists every `.json` file with `include_str!` so the registry
/// can load them without a hardcoded list.
fn generate_builtin_themes(manifest_dir: &PathBuf, out_dir: &PathBuf) {
    let light_dir = manifest_dir.join("assets/themes/light");
    let dark_dir = manifest_dir.join("assets/themes/dark");

    let mut light_entries = Vec::new();
    let mut dark_entries = Vec::new();

    if let Ok(entries) = fs::read_dir(&light_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let name = path.file_stem().unwrap().to_string_lossy();
                let rel = path.strip_prefix(manifest_dir).unwrap();
                let rel_str = rel.to_string_lossy().replace('\\', "/");
                light_entries.push((name.to_string(), rel_str));
            }
        }
    }
    light_entries.sort_by(|a, b| a.0.cmp(&b.0));

    if let Ok(entries) = fs::read_dir(&dark_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let name = path.file_stem().unwrap().to_string_lossy();
                let rel = path.strip_prefix(manifest_dir).unwrap();
                let rel_str = rel.to_string_lossy().replace('\\', "/");
                dark_entries.push((name.to_string(), rel_str));
            }
        }
    }
    dark_entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut out = String::new();
    out.push_str("const BUILTIN_LIGHT: &[(&str, &str)] = &[\n");
    for (name, rel) in &light_entries {
        out.push_str(&format!(
            r#"    ("{name}", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/{rel}"))),"#,
            name = name,
            rel = rel
        ));
        out.push('\n');
    }
    out.push_str("];\n\n");

    out.push_str("const BUILTIN_DARK: &[(&str, &str)] = &[\n");
    for (name, rel) in &dark_entries {
        out.push_str(&format!(
            r#"    ("{name}", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/{rel}"))),"#,
            name = name,
            rel = rel
        ));
        out.push('\n');
    }
    out.push_str("];\n");

    let out_path = out_dir.join("builtin_themes.rs");
    fs::write(&out_path, out).expect("Failed to write builtin_themes.rs");

    println!("cargo:rerun-if-changed=assets/themes");
    println!("cargo:warning=Generated {} light and {} dark built-in themes", light_entries.len(), dark_entries.len());
}
