use std::io::{Result};
use std::net::{SocketAddrV4, TcpListener, TcpStream};
use std::{io, thread};
use std::sync::Arc;
use clap::Parser;
use log::LevelFilter;

#[macro_use]
extern crate log;

const LISTENER_PORT_DEFAULT: u16 = 21883;

/// a simple tcp proxy service
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// local listen port, default 21883, example: 31883
    #[clap(short, long)]
    listen: Option<u16>,

    /// target server address, example: 192.168.0.10:1883
    #[clap(short, long)]
    target: String,
}


fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let args: Args = Args::parse();
    let port = args.listen.unwrap_or(LISTENER_PORT_DEFAULT);
    let target_addr: SocketAddrV4 = args.target.parse()
        .expect("Invalid Socket address");
    let listen_addr = "127.0.0.1:".to_string() + &port.to_string();

    info!("rs-proxy starting... ");
    info!("listen to {}, target address is {}", listen_addr, target_addr);

    match start_service(listen_addr, target_addr) {
        Err(e) => {
            error!("Start rs-proxy failed. {}", e)
        }
        _ => {}
    }
}

fn start_service(listen_addr: String, target_addr: SocketAddrV4) -> Result<()> {
    let listener = TcpListener::bind(&listen_addr)
        .expect("Failed start Listener");
    let target_stream = TcpStream::connect(target_addr)
        .expect("Failed connect target host.");
    info!("rs-proxy started.");

    for listen_stream in listener.incoming() {
        handle_client(listen_stream?, &target_stream)?;
    }
    Ok(())
}

fn handle_client(listen_stream: TcpStream, target_stream: &TcpStream) -> Result<()> {
    info!("Connection established from {}", listen_stream.peer_addr()?);
    info!("Connection established by {}", &target_stream.peer_addr()?);

    let listen_arc = Arc::new(listen_stream);
    let target_arc = Arc::new(target_stream);

    let (mut listen_rx, mut listen_tx) = (listen_arc.try_clone()?, listen_arc.try_clone()?);
    let (mut target_rx, mut target_tx) = (target_arc.try_clone()?, target_arc.try_clone()?);

    let connections = vec![
        thread::spawn(move || io::copy(&mut listen_rx, &mut target_tx).unwrap()),
        thread::spawn(move || io::copy(&mut target_rx, &mut listen_tx).unwrap()),
    ];

    for handle in connections {
        handle.join().unwrap();
    }
    Ok(())
}

