use std::collections::HashSet;
use clap::Parser;
use serde_derive::Deserialize;

/// a simple tcp proxy service
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct RsProxyArgs {
    /// rs-proxy.toml file
    #[clap(short, long)]
    pub config: Option<String>,
}



#[derive(Deserialize)]
#[derive(Debug)]
pub struct RsProxyConfig
{
    pub proxy: Option<Vec<ProxyConfig>>,
}

#[derive(Deserialize)]
#[derive(Debug, )]
pub struct ProxyConfig {
    pub enable: Option<bool>,
    pub name: String,
    pub listen: u16,
    pub target: String,
    pub allow_list: Option<HashSet<String>>,
}