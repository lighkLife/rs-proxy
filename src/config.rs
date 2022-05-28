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
#[derive(Debug)]
pub struct ProxyConfig {
    pub name: Option<String>,
    pub listen: u16,
    pub target: String,
}