use std::{io, thread};
use std::collections::HashSet;
use std::net::{Ipv4Addr, Shutdown, SocketAddrV4, TcpListener, TcpStream};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::thread::JoinHandle;
use thread::spawn;

use anyhow::{bail, Result};
use log::{error, info};

use crate::ProxyConfig;

pub struct ProxyService {
    name: Arc<String>,
    listen: Arc<SocketAddrV4>,
    target: Arc<SocketAddrV4>,
    allow_list: Arc<HashSet<String>>,
    connection: Arc<Option<Connection>>,
}

pub struct Connection {
    name: Arc<String>,
    listen_stream: Arc<TcpStream>,
    target_stream: Arc<TcpStream>,
    rx_bytes: AtomicU32,
    tx_bytes: AtomicU32,
}

impl Connection {
    pub fn new(pre: Arc<Option<Connection>>, name: Arc<String>, listen_stream: TcpStream, target: Arc<SocketAddrV4>) -> Result<Connection> {
        let target_stream = TcpStream::connect(*target)
            .map_err(|error| {
                error!("{} failed to connect target {}",name, target);
                let _ = listen_stream.shutdown(Shutdown::Both);
                return error;
            })?;
        info!("{} connection established from {}", name, listen_stream.peer_addr()?);
        info!("{} connection established to {}", name, target_stream.peer_addr()?);
        let mut con = Connection {
            name,
            listen_stream: Arc::new(listen_stream),
            target_stream: Arc::new(target_stream),
            rx_bytes: AtomicU32::new(0),
            tx_bytes: AtomicU32::new(0),
        };
        return Ok(con);
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let name = self.name.clone();
        // close previous steam
        self.target_stream.shutdown(Shutdown::Both).unwrap();
        info!("{} shutdown previous source stream {:?}", name.clone(), self.target_stream.clone());

        // close previous steam
        self.listen_stream.shutdown(Shutdown::Both).unwrap();
        info!("{} shutdown previous source stream {:?}", name.clone(), self.listen_stream.clone());
    }
}


impl ProxyService {
    pub fn new(config: Rc<ProxyConfig>) -> Result<ProxyService> {
        let listen_socket = SocketAddrV4::new(Ipv4Addr::from_str("0.0.0.0")?, config.listen);
        let target_socket = SocketAddrV4::from_str(config.target.as_str())?;
        Ok(ProxyService {
            name: Arc::new(config.name.clone()),
            listen: Arc::new(listen_socket),
            target: Arc::new(target_socket),
            allow_list: Arc::new(config.allow_list.clone().unwrap_or_default()),
            connection: Arc::new(None),
        })
    }

    pub fn run(mut self) -> JoinHandle<()> {
        let name = self.name.clone();
        let listen = self.listen.clone();
        let target = self.target.clone();
        let allow_list = self.allow_list.clone();
        info!("{:?} listen: {}, target:{} ",self.name, listen, target);
        spawn(move || {
            let listener = TcpListener::bind(*listen)
                .expect("Failed start Listener");
            for incoming in listener.incoming() {
                let pre_connection = self.connection.clone();
                let name = name.clone();
                let result = match incoming {
                    Ok(stream) => {
                        match deny_if_need(allow_list.clone(), &stream) {
                            Ok(_) => {
                                let connection = Connection::new(pre_connection, name.clone(), stream, target.clone());
                                if connection.is_ok() {
                                    let connection = Arc::new(connection.unwrap());
                                    let e = transfer_stream(connection.clone());
                                    if e.is_err() {
                                        error!("{} copy between listen and target failed, {:?}", name.clone(), e);
                                    }
                                } else {
                                    error!("{} connection create failed, {:?}", name.clone(), connection.err().unwrap())
                                }
                            }
                            Err(e) => error!("{} deny connect, {:?}", name.clone(), e)
                        }
                    }
                    Err(e) => {
                        error!("{} connect source failed, {:?}", name.clone(), e);
                    }
                };
            }
        })
    }
}

fn deny_if_need(allow_list: Arc<HashSet<String>>, stream: &TcpStream) -> Result<()> {
    if !allow_list.is_empty() {
        let peer_ip = stream.peer_addr()
            .map(|it| it.ip().to_string())?;
        let contains = allow_list.contains(peer_ip.as_str());
        if !contains {
            bail!("deny connect from {}, because it not in allow list.", peer_ip);
        }
    }
    Ok(())
}

fn transfer_stream(con: Arc<Connection>) -> Result<()> {
    let request = con.clone();
    let response = con.clone();

    let name = con.name.clone();
    spawn(move || {
        let mut listen = request.listen_stream.as_ref();
        let mut target = request.target_stream.as_ref();
        match io::copy(&mut listen, &mut target) {
            Err(e) => error!("{} copy request failed. {:?}", name.clone(), e),
            _ => {}
        }
    });
    let name = con.name.clone();
    spawn(move || {
        let mut listen = response.listen_stream.as_ref();
        let mut target = response.target_stream.as_ref();
        match io::copy(&mut target, &mut listen) {
            Err(e) => error!("{} copy response failed. {:?}", name.clone(), e),
            _ => {}
        }
    });
    Ok(())
}


