extern crate core;
#[macro_use]
extern crate log;
extern crate serde_derive;
extern crate toml;

use clap::Parser;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use anyhow::{Context, Result};

use log::LevelFilter;
use rs_proxy::{ProxyService, RsProxyArgs, RsProxyConfig};

pub static CONFIG_EXAMPLE: &str = r#"
[[proxy]]
name = "example"
listen = 21883
target = "127.0.0.1:1883"
"#;

fn main() {
    env_logger::builder().filter_level(LevelFilter::Debug).init();

    let args: RsProxyArgs = RsProxyArgs::parse();
    let config_value = read_config(args.config).unwrap();
    debug!("rs-proxy config: \n{}", config_value);
    info!("rs-proxy starting... ");
    let rs_proxy_config: RsProxyConfig = toml::from_str(&config_value).unwrap();
    let mut handles = Vec::new();
    match rs_proxy_config.proxy {
        Some(proxy_vec) => {
            for proxy in proxy_vec {
                let service = ProxyService::new(proxy.name, proxy.listen, proxy.target).unwrap();
                handles.push(service.run());
            }
        }
        None => info!("rs-proxy exit with empty config.")
    };
    for handle in handles {
        handle.join().expect("exit.");
    }
}

fn read_config(config_path: Option<String>) -> Result<String> {
    let mut config_value = String::new();
    match config_path {
        None => {
            let path = Path::new("rs-proxy.toml");
            if path.exists() {
                let mut file = File::open(path)?;
                file.read_to_string(&mut config_value)?;
            } else {
                let mut file = File::create(path)?;
                file.write(CONFIG_EXAMPLE.as_bytes())?;
                config_value = CONFIG_EXAMPLE.to_string();
            }
        }
        Some(path) => {
            let mut file = File::open(path)?;
            file.read_to_string(&mut config_value)?;
        }
    }
    return Ok(config_value);
}


