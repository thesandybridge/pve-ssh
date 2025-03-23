mod config;

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::AppConfig;
use dialoguer::{theme::ColorfulTheme, Select};
use std::{path::PathBuf, process::Command};

#[derive(Parser)]
#[command(name = "pve-ssh", version, about = "SSH into your Proxmox VMs easily")]
struct Cli {
    #[arg(short, long, help = "Path to the config file")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    List,
    Connect {
        #[arg(help = "Name of the VM")]
        name: String,
    },
}

fn main() -> Result<()> {
    ctrlc::set_handler(move || {
        println!("\n❌ Interrupted (Ctrl-C)");
        std::process::exit(130);
    })?;
    let cli = Cli::parse();
    let config_path = cli.config.unwrap_or_else(config::get_default_path);
    let app_config: AppConfig = config::load_config(config_path)?;

    match cli.command {
        Some(Commands::List) => {
            for vm in &app_config.vms {
                println!("{} => {}@{}", vm.name, vm.user, vm.ip);
            }
        }

        Some(Commands::Connect { name }) => {
            if let Some(vm) = app_config.vms.iter().find(|vm| vm.name == name) {
                println!("Connecting to {}@{}...", vm.user, vm.ip);
                Command::new("ssh")
                    .arg(format!("{}@{}", vm.user, vm.ip))
                    .status()?;
            } else {
                eprintln!("VM '{}' not found.", name);
            }
        }

        None => {
            let vm_labels: Vec<String> = app_config
                .vms
                .iter()
                .map(|vm| format!("{} ({}@{})", vm.name, vm.user, vm.ip))
                .collect();

            let theme = ColorfulTheme::default();

            let selected = Select::with_theme(&theme)
                .with_prompt("Select a VM")
                .items(&vm_labels)
                .default(0)
                .interact_opt()?; // <- safe on Ctrl-D / Esc

            if let Some(index) = selected {
                let vm = &app_config.vms[index];
                println!("Connecting to {}@{}...", vm.user, vm.ip);
                Command::new("ssh")
                    .arg(format!("{}@{}", vm.user, vm.ip))
                    .status()?;
            } else {
                println!("❌ Cancelled.");
            }
        }
    }

    Ok(())
}
