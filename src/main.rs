use std::{
    env, fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use gcpdyndns::{
    dns::edit_dns_record,
    error::{Error, Result},
    external::get_external_ip,
    persistent::Persistance,
};
use log::{LevelFilter, warn};
use rstaples::{display::printkv, logging::StaplesLogger};

const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct UserArgs {
    /// auth JSON file
    #[arg(short, long, default_value_t = default_key())]
    auth: String,

    /// GCP Project
    #[arg(short, long)]
    project: String,

    /// GCP DNS Zone
    #[arg(short, long)]
    zone: String,

    /// DNS name
    #[arg(short, long)]
    name: String,

    /// force update
    #[arg(short, long)]
    force: bool,

    /// verbose
    #[arg(short, long)]
    verbose: bool,
}

fn default_key() -> String {
    if let Ok(self_exe) = env::current_exe() {
        //
        // is there a config.toml SxS to the executable
        //
        if let Some(self_dir) = self_exe.parent() {
            let config_file = self_dir.join("key.json");
            return config_file.to_string_lossy().to_string();
        }
    }

    "".into()
}

fn get_data_dir() -> Result<PathBuf> {
    let data_root = dirs::data_dir().ok_or(Error::DataDirNotFound)?;

    let data_dir = data_root.join(CRATE_NAME);

    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)?;
    }

    Ok(data_dir)
}

fn install_auth(sa_file: &Path) -> Result<()> {
    let sa_file_str = sa_file.to_string_lossy().to_string();

    unsafe { env::set_var("GOOGLE_APPLICATION_CREDENTIALS", sa_file_str) }

    Ok(())
}

async fn update(mut persistent: Persistance, args: UserArgs) -> Result<()> {
    let ip_addr = get_external_ip().await?;

    let changed = persistent.ip_changed(&ip_addr);

    if changed || args.force {
        warn!("new ip {ip_addr}");
        edit_dns_record(&args.project, &args.zone, &args.name, &ip_addr).await?;
        persistent.update(ip_addr)?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = UserArgs::parse();

    let log_level = match args.verbose {
        true => LevelFilter::Info,
        false => LevelFilter::Warn,
    };

    StaplesLogger::new()
        .with_colors()
        .with_log_level(log_level)
        .start();

    let auth_file = PathBuf::from(&args.auth).canonicalize()?;

    let data_dir = get_data_dir()?;

    install_auth(&auth_file)?;

    if args.verbose {
        println!("GCPDynDns:");
        printkv("Auth File", auth_file.display());
        printkv("Project", &args.project);
        printkv("Zone", &args.zone);
        printkv("DNS Name", &args.name);
        printkv("Data Directory", data_dir.display());
    }

    let persistent = Persistance::new(data_dir, &args.name)?;

    update(persistent, args).await
}
