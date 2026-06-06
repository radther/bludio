use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Only attempt policy installation on Linux
    if env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "linux" {
        return;
    }

    let policy_src = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("policy")
        .join("dev.toomosin.bludio.policy");

    if !policy_src.exists() {
        println!("cargo:warning=PolicyKit policy file not found at {:?}", policy_src);
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
                    println!("cargo:warning=Insufficient privileges to write to {:?}. PolicyKit policy was NOT installed.", policy_dir);
                    println!("cargo:warning=To enable privileged Bluetooth operations, run one of the following:");
                    println!("cargo:warning=  sudo cargo install --git https://github.com/toomosin/bludio");
                    println!("cargo:warning=  sudo cp policy/dev.toomosin.bludio.policy /usr/share/polkit-1/actions/");
                    return;
                }
            }
        }
        _ => {
            println!("cargo:warning=PolicyKit actions directory {:?} does not exist. Policy was NOT installed.", policy_dir);
            println!("cargo:warning=To enable privileged Bluetooth operations, manually copy the policy file:");
            println!("cargo:warning=  sudo mkdir -p /usr/share/polkit-1/actions/");
            println!("cargo:warning=  sudo cp policy/dev.toomosin.bludio.policy /usr/share/polkit-1/actions/");
            return;
        }
    }

    // Copy the policy file
    match fs::copy(&policy_src, &policy_dest) {
        Ok(_) => {
            println!("cargo:warning=PolicyKit policy installed to {:?}", policy_dest);
        }
        Err(e) => {
            println!("cargo:warning=Failed to install PolicyKit policy to {:?}: {}", policy_dest, e);
            println!("cargo:warning=To enable privileged Bluetooth operations, manually copy the policy file:");
            println!("cargo:warning=  sudo cp policy/dev.toomosin.bludio.policy /usr/share/polkit-1/actions/");
        }
    }
}
