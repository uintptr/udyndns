use std::{env, path::PathBuf};

use anyhow::Result;
use clap::Args;

use reqwest::Client;
use serde::Serialize;

use log::error;

use crate::{error::Error, external::ExternalIp};

const DNS_API_URL: &str = "https://dns.googleapis.com/dns/v1beta2/projects";

#[derive(Args, Clone)]
pub struct GcpArgs {
    /// auth JSON file
    #[arg(short, long)]
    pub auth_file: PathBuf,

    /// GCP Project
    #[arg(short, long)]
    pub project: String,

    /// GCP DNS Zone
    #[arg(short, long)]
    pub zone: String,

    /// hostname name
    #[arg(short = 'H', long)]
    pub hostname: String,
}

#[derive(Debug, Serialize)]
struct DnsPatchRequest<'a> {
    rrdatas: Vec<&'a str>,
}

async fn auth() -> Result<String> {
    let provider = gcp_auth::provider().await?;
    let scopes = &["https://www.googleapis.com/auth/cloud-platform"];
    let token = provider.token(scopes).await?;
    Ok(token.as_str().to_string())
}

impl GcpArgs {
    fn install_auth(&self) -> Result<()> {
        let sa_file_str = self.auth_file.to_string_lossy().to_string();

        unsafe { env::set_var("GOOGLE_APPLICATION_CREDENTIALS", sa_file_str) }

        Ok(())
    }

    async fn edit_dns_record(&self, ip: &ExternalIp) -> Result<()> {
        let token = auth().await?;

        //
        // make sure the name ends with a dot (.)
        //
        let name = match self.hostname.ends_with(".") {
            true => self.hostname.to_string(),
            false => format!("{}.", self.hostname),
        };

        let rec_type = if ip.is_ipv4() { "A" } else { "AAAA" };

        let url = format!(
            "{}/{}/managedZones/{}/rrsets/{name}/{}",
            DNS_API_URL, self.project, self.zone, rec_type
        );

        let req_data = DnsPatchRequest {
            rrdatas: vec![&ip.address],
        };

        let client = Client::new();

        let res = client
            .patch(&url)
            .header("Content-Type", "application/json")
            .bearer_auth(token)
            .json(&req_data)
            .send()
            .await?;

        if res.status().is_success() {
            Ok(())
        } else {
            error!("{url} returned {}", res.status().as_str());
            Err(Error::UpdateFailure(res.status()).into())
        }
    }

    pub async fn update(&self, ip: &ExternalIp) -> Result<()> {
        if let Err(e) = self.install_auth() {
            error!("unable to install auth ({e})");
            return Err(e.into());
        }

        self.edit_dns_record(ip).await
    }
}
