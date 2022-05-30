use std::net::{Ipv4Addr, Shutdown, SocketAddrV4, TcpListener, TcpStream};
use std::str::FromStr;
use std::{io, thread};
use std::sync::Arc;
use std::thread::JoinHandle;

use anyhow::{Result};
use thread::spawn;
use log::{error, info};

pub struct ProxyService {
    name: Arc<String>,
    listen: Arc<SocketAddrV4>,
    target: Arc<SocketAddrV4>,
}


impl ProxyService {
    pub fn new(name: String, listen: u16, target: String) -> Result<ProxyService> {
        let listen_socket = SocketAddrV4::new(Ipv4Addr::from_str("0.0.0.0")?, listen);
        let target_socket = SocketAddrV4::from_str(target.as_str())?;
        Ok(ProxyService {
            name: Arc::new(name),
            listen: Arc::new(listen_socket),
            target: Arc::new(target_socket),
        })
    }

    pub fn run(self) -> JoinHandle<()> {
        let name = self.name.clone();
        let listen = self.listen.clone();
        let target = self.target.clone();
        info!("{:?} listen: {}, target:{} ",self.name, listen, target);
        spawn(move || {
            let listener = TcpListener::bind(*listen)
                .expect("Failed start Listener");
            for incoming in listener.incoming() {
                let _ = hand_client(name.clone(), incoming, target.clone())
                    .map_err(|e| {
                        error!("{} hand_client error, {:?}", name.clone(),  e);
                    });
            }
        })
    }
}

fn hand_client(name: Arc<String>, incoming: io::Result<TcpStream>, target: Arc<SocketAddrV4>) -> Result<()> {
    let listen_stream = incoming?;
    let target_stream = TcpStream::connect(*target)
        .map_err(|error| {
            error!("{} failed to connect target {}",name, target);
            let _ = listen_stream.shutdown(Shutdown::Both);
            return error;
        })?;
    info!("{} connection established from {}", name, listen_stream.peer_addr()?);
    info!("{} connection established to {}", name, target_stream.peer_addr()?);
    transfer_stream(name, listen_stream, target_stream)?;
    Ok(())
}

fn transfer_stream(name: Arc<String>, listen_stream: TcpStream, target_stream: TcpStream) -> Result<()> {
    let name_copy = name.clone();

    let listen_arc = Arc::new(listen_stream);
    let target_arc = Arc::new(target_stream);

    let (mut listen_rx, mut listen_tx) = (listen_arc.try_clone()?, listen_arc.try_clone()?);
    let (mut target_rx, mut target_tx) = (target_arc.try_clone()?, target_arc.try_clone()?);

    spawn(move || {
        match io::copy(&mut listen_rx, &mut target_tx) {
            Err(e) => error!("{} copy client request to target failed. {:?}", name, e),
            _ => {}
        }
    });
    spawn(move || {
        match io::copy(&mut target_rx, &mut listen_tx) {
            Err(e) => error!("{} copy target response to client failed. {:?}", name_copy, e),
            _ => {}
        }
    });
    Ok(())
}
