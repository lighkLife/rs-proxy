use std::net::{AddrParseError, Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::str::FromStr;
use std::{io, thread};
use std::sync::Arc;
use std::thread::JoinHandle;
use thread::spawn;
use log::info;

pub struct ProxyService {
    name: Arc<Option<String>>,
    listen: Arc<SocketAddrV4>,
    target: Arc<SocketAddrV4>,
}

impl ProxyService {
    pub fn new(name: Option<String>, listen: u16, target: String) -> Result<ProxyService, AddrParseError> {
        let listen_socket = SocketAddrV4::new(Ipv4Addr::from_str("127.0.0.1")?, listen);
        let target_socket = SocketAddrV4::from_str(target.as_str())?;
        Ok(ProxyService {
            name: Arc::new(name),
            listen: Arc::new(listen_socket),
            target: Arc::new(target_socket),
        })
    }

    pub fn run(self) -> JoinHandle<()> {
        let listen = self.listen.clone();
        let target = self.target.clone();
        info!("rs-proxy {:?} listen: {}, target:{} ",self.name, listen, target);
        spawn(move || {
            let listener = TcpListener::bind(*listen)
                .expect("Failed start Listener");
            for incoming in listener.incoming() {
                hand_client(incoming, target.clone());
            }
        })
    }
}

fn hand_client(incoming: io::Result<TcpStream>, target: Arc<SocketAddrV4>) {
    let listen_stream = incoming
        .expect("Failed connect target host.");
    let target_stream = TcpStream::connect(*target)
        .expect("Failed connect target host.");
    info!("Connection established from {}", listen_stream.peer_addr().unwrap());
    info!("Connection established to {}", &target_stream.peer_addr().unwrap());
    transfer_stream(listen_stream, target_stream)
        .unwrap();
}

fn transfer_stream(listen_stream: TcpStream, target_stream: TcpStream) -> io::Result<()> {
    let listen_arc = Arc::new(listen_stream);
    let target_arc = Arc::new(target_stream);

    let (mut listen_rx, mut listen_tx) = (listen_arc.try_clone()?, listen_arc.try_clone()?);
    let (mut target_rx, mut target_tx) = (target_arc.try_clone()?, target_arc.try_clone()?);

    spawn(move || io::copy(&mut listen_rx, &mut target_tx).unwrap());
    spawn(move || io::copy(&mut target_rx, &mut listen_tx).unwrap());
    Ok(())
}
