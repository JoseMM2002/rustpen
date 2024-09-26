use dirs;
use fs_extra::dir::{copy, CopyOptions};
use std::fs::remove_dir_all;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

const CONFIG_FILE: &str = "config.toml";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let current_dir_config = PathBuf::from(CONFIG_FILE);
    if !current_dir_config.exists() {
        panic!("The file ./{} does not exist", CONFIG_FILE);
    }

    let config_dir = dirs::config_dir().expect("Failed to get the config directory");
    let rustpen_config_dir = config_dir.join("rustpen");

    if !rustpen_config_dir.exists() {
        fs::create_dir_all(&rustpen_config_dir)
            .expect("Failed to create ~/.config/rustpen directory");
        println!("cargo:warning=Created directory ~/.config/rustpen.");
    }

    let config_dir = "/tmp/rustpen_unix_socket/";

    if !Path::new(&config_dir).exists() {
        fs::create_dir_all(&config_dir).expect("Failed to create /tmp/rustpen_unix_socket/");
        println!("cargo:warning=Created binding server in /tmp/rustpen_unix_socket/")
    }

    let destination_file = rustpen_config_dir.join(CONFIG_FILE);

    fs::copy(&current_dir_config, &destination_file).expect(&format!(
        "Failed to copy ./{} to ~/.config/rustpen/{}",
        CONFIG_FILE, CONFIG_FILE
    ));

    println!(
        "cargo:warning=Successfully copied {} to ~/.config/rustpen/{}.",
        CONFIG_FILE, CONFIG_FILE
    );

    let shell_path = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let shell_name = shell_path.split('/').last().unwrap_or("bash");

    println!("cargo:warning=Detected shell: {}", shell_name);

    let profile_path = match shell_name {
        "bash" => "~/.bashrc",
        "zsh" => "~/.zshrc",
        _ => "~/.profile",
    };

    let nvm_check = Command::new(&shell_path)
        .arg("-c")
        .arg(format!("source {} && command -v nvm", profile_path))
        .output()
        .expect("Failed to check for NVM");

    if nvm_check.stdout.is_empty() {
        println!("cargo:warning=NVM is not installed. Installing NVM...");

        Command::new(&shell_path)
            .arg("-c")
            .arg("curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.3/install.sh | bash")
            .status()
            .expect("Failed to install NVM");

        Command::new(&shell_path)
            .arg("-c")
            .arg("source ~/.nvm/nvm.sh")
            .status()
            .expect("Failed to source NVM");
    } else {
        println!("cargo:warning=NVM is already installed.");
    }

    Command::new(&shell_path)
        .arg("-c")
        .arg("nvm install --lts")
        .status()
        .expect("Failed to install LTS");

    let ts_client_node_modules = "./ts-client/node_modules";
    if Path::new(ts_client_node_modules).exists() {
        println!("cargo:warning=Found node_modules in ./ts-client/. Removing it...");
        remove_dir_all(ts_client_node_modules)
            .expect("Failed to remove node_modules from ts-client.");
        println!("cargo:warning=node_modules removed successfully from ./ts-client/.");
    }

    let home_dir = env::var("HOME").expect("HOME directory not found");
    let config_dir = format!("{}/.config/rustpen", home_dir);

    if !Path::new(&config_dir).exists() {
        fs::create_dir_all(&config_dir).expect("Failed to create rustpen directory");
        println!("cargo:warning=Created directory: {}", config_dir);
    } else {
        println!("cargo:warning=Directory {} already exists.", config_dir);
    }

    let ts_client_dir = "ts-plugins/";

    if Path::new(ts_client_dir).exists() {
        let mut options = CopyOptions::new();
        options.overwrite = false;
        options.skip_exist = true;

        copy(ts_client_dir, &config_dir, &options)
            .expect("Failed to copy ts-client skeleton to ts-plugins");

        println!("cargo:warning=Copied ./ts-plugins to ~/.config/rustpen/ts-plugins/ (excluding node_modules)");
    } else {
        println!("cargo:warning=The directory ./ts-plugins does not exist.");
    }

    let config_dir = format!("{}/ts-plugins/", config_dir);

    println!("cargo:warning=Running npm install to install project dependencies...");
    Command::new(&shell_path)
        .current_dir(&config_dir)
        .arg("-c")
        .arg(format!("source {} && npm install", profile_path))
        .status()
        .expect("Failed to run npm install");

    Command::new(&shell_path)
        .current_dir(&config_dir)
        .arg("-c")
        .arg("npm i @rustpen/client@latest")
        .status()
        .expect("Failed to update client package");

    println!("cargo:warning=Dependencies installed successfully.");
}
