use std::io::{BufReader, BufWriter, Read};
use std::net::{IpAddr, SocketAddrV4, TcpListener, TcpStream};
use std::str::FromStr;
use clap::Parser;

#[macro_use]
extern crate log;

/// proxy
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// listen port
    #[clap(short, long)]
    port: Option<u16>,

    /// forward to
    #[clap(short, long)]
    forward: String,
}

fn main() {
    env_logger::init();

    let args :Args = Args::parse();
    let port = args.port.unwrap_or(20080);
    let addr: SocketAddrV4 = args.forward.parse().unwrap();

    info!("Start rs-proxy... ");
    info!("listen {}, forward {}", port, addr);

    match start_service(port, addr) {
        Err(e) => {
            error!("Start rs-proxy failed. {}", e)
        }
        _ => {}
    }

}

fn start_service(port: u16, addr: SocketAddrV4) -> std::io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1" + port)?;
    for stream in listener.incoming() {
        handle_client(stream?);
    }
    Ok(())
}

fn handle_client(stream: TcpStream) {
    let peer = stream.peer_addr()?;
    info!("Connection established from {}", &peer);
    let reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
}

