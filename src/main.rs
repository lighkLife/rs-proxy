extern crate core;
#[macro_use]
extern crate log;

use io::{BufReader, Lines};
use std::{io, thread};
use std::collections::HashSet;
use std::fmt::Debug;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufRead, Result};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::path::Path;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use std::thread::JoinHandle;

use clap::Parser;
use log::LevelFilter;

/// a simple tcp proxy service
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// rs-proxy.config file
    #[clap(short, long, default_value = "./rs-proxy.config")]
    config: String,
}

#[derive(PartialEq, Eq, Hash)]
struct PoxyAddr {
    listen: Arc<SocketAddrV4>,
    target: Arc<SocketAddrV4>,
}


impl PoxyAddr {
    pub fn new(listen: SocketAddrV4, target: SocketAddrV4) -> PoxyAddr {
        let listen = Arc::new(listen);
        let target = Arc::new(target);
        PoxyAddr { listen, target }
    }
}


fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let args: Args = Args::parse();
    let proxy_list = read_lines(args.config)
        .and_then(|lines| read_proxy_addr(lines))
        .expect("Read config failed.");

    info!("rs-proxy starting... ");
    let mut connections = Vec::new();
    for proxy in proxy_list.iter() {
        info!("listen to {}, target address is {}", proxy.listen, proxy.target);
        connections.push(thread::spawn(move || start_service(*proxy.listen.clone(), *proxy.target.clone())));
    }
    info!("rs-proxy started.");
    for con in connections {
        con.join().unwrap();
    }
}

fn read_proxy_addr(lines: Lines<BufReader<File>>) -> Result<HashSet<PoxyAddr>> {
    let localhost = Ipv4Addr::from_str("0.0.0.0").unwrap();
    let mut proxy_list = HashSet::new();
    for line in lines {
        if let Ok(proxy_addr) = line {
            if proxy_addr.starts_with("#") {
                continue;
            }
            let collect = proxy_addr.split("->").collect::<Vec<&str>>();
            if collect.len() != 2 {
                error!("Invalid config. {}", proxy_addr);
                panic!("exit");
            }
            let listen = collect.get(0).unwrap().trim();
            let target = collect.get(1).unwrap().trim();
            let listen_socket_addr = SocketAddrV4::new(localhost, listen.parse().unwrap());
            let target_socket_addr = SocketAddrV4::from_str(target).unwrap();
            proxy_list.insert(PoxyAddr::new(listen_socket_addr, target_socket_addr));
        }
    }
    return Ok(proxy_list);
}

fn read_lines<P>(filename: P) -> Result<Lines<BufReader<File>>>
    where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
}

fn start_service(listen_addr: SocketAddrV4, target_addr: SocketAddrV4) {
    let listener = TcpListener::bind(&listen_addr)
        .expect("Failed start Listener");

    for listen_stream in listener.incoming() {
        handle_client(listen_stream.unwrap(), target_addr)
            .expect("error occurred in listener")
    }
}

fn handle_client(listen_stream: TcpStream, target_addr: SocketAddrV4) -> Result<()> {
    let target_stream = TcpStream::connect(target_addr)
        .expect("Failed connect target host.");
    info!("Connection established from {}", listen_stream.peer_addr()?);
    info!("Connection established to {}", &target_stream.peer_addr()?);

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

