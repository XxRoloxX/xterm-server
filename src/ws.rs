use super::pty::Pty;
use log::{error, info};
use std::net::TcpStream;
use std::os::fd::OwnedFd;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use websocket::server::upgrade::WsUpgrade;
use websocket::server::{InvalidConnection, NoTlsAcceptor};
use websocket::sync::server::upgrade::Buffer;
use websocket::sync::server::Server as WsServer;
use websocket::sync::{Reader, Server, Writer};
use websocket::Message;
use websocket::OwnedMessage;

const WEBSOCKET_BIND_ADDR: &str = "127.0.0.1:3030";

pub fn sync_websockets() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = bind_to_ws_socket()?;

    loop {
        let connection = match wait_for_ws_connection(&mut server) {
            Ok(connection) => connection,
            Err(e) => {
                error!("Error while waiting for connection: {:?}", e);
                continue;
            }
        };

        let (tx, rx) = mpsc::channel::<String>();
        let shared_rx = Arc::from(Mutex::from(rx));
        let shared_tx = tx.clone();
        let pty = Pty::new();

        let (receiver, sender) = match unpack_ws_connection(connection) {
            Ok((reciever, sender)) => (reciever, sender),
            Err(e) => {
                error!("Error while unpacking connection: {:?}", e);
                continue;
            }
        };

        let pty_fd = match pty.master_fd.try_clone() {
            Ok(fd) => fd,
            Err(e) => {
                error!("Error while cloning pty fd: {:?}", e);
                continue;
            }
        };

        let _handle_pty_output_thread = handle_pty_output(shared_tx, pty);
        let _handle_mpsc_data_thread = handle_mpsc_data(shared_rx.clone(), sender);
        let _handle_websocket_message_thread = handle_websocket_message(receiver, pty_fd);
    }
}

pub fn bind_to_ws_socket() -> Result<WsServer<NoTlsAcceptor>, Box<dyn std::error::Error>> {
    let server = Server::bind(WEBSOCKET_BIND_ADDR).map_err(|e| {
        error!("Couldn't bind to a socket {}:{}", WEBSOCKET_BIND_ADDR, e);
        e
    })?;

    Ok(server)
}

pub fn handle_pty_output(sender: mpsc::Sender<String>, pty: Pty) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let handle_pty_data = Box::new(move |data: &str| match sender.send(data.to_string()) {
            Ok(_) => (),
            Err(e) => {
                error!("Error sending data to channel: {:?}", e);
            }
        });

        pty.listen(handle_pty_data);
    })
}

pub fn handle_mpsc_data(
    receiver: Arc<Mutex<mpsc::Receiver<String>>>,
    mut ws_sender: websocket::sender::Writer<std::net::TcpStream>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || loop {
        let receiver = receiver.lock();

        if receiver.is_err() {
            error!("Error locking receiver: {:?}", receiver);
            break;
        }

        let message = receiver.unwrap().recv();

        if let Err(e) = message {
            error!("Error receiving message from channel: {:?}", e);
            break;
        }

        info!("Received message from channel: {:?}", message);

        if let Err(err) = ws_sender.send_message(&Message::text(message.unwrap())) {
            info!("Error sending message to websocket: {:?}", err);
            break;
        }
    })
}
pub fn handle_websocket_message(
    mut receiver: websocket::receiver::Reader<std::net::TcpStream>,
    pty_fd: OwnedFd,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while let Ok(msg) = receiver.recv_message() {
            match msg {
                OwnedMessage::Text(msg) => {
                    let result = nix::unistd::write(&pty_fd, msg.as_bytes());
                    if result.is_err() {
                        error!("Error writing to pty: {:?}", msg);
                    }
                }
                OwnedMessage::Binary(msg) => {
                    let result = nix::unistd::write(&pty_fd, &msg);
                    if result.is_err() {
                        error!("Error writing to pty: {:?}", msg);
                    }
                }
                msg => {
                    info!("Received non-text message: {:?}", msg);
                }
            }
        }
    })
}
pub fn wait_for_ws_connection(
    server: &mut WsServer<NoTlsAcceptor>,
) -> Result<WsUpgrade<TcpStream, Option<Buffer>>, InvalidConnection<TcpStream, Buffer>> {
    match server.accept() {
        Ok(ws_upgrade) => {
            info!("Connection from {:?} accepted", ws_upgrade.request);
            Ok(ws_upgrade)
        }
        Err(e) => {
            error!("Error while accepting connection: {:?}", e);
            Err(e)
        }
    }
}

pub fn unpack_ws_connection(
    connection: WsUpgrade<TcpStream, Option<Buffer>>,
) -> Result<(Reader<TcpStream>, Writer<TcpStream>), Box<dyn std::error::Error>> {
    let new_connection = connection.accept();
    match new_connection {
        Ok(new_connection) => {
            info!("Connection accepted");
            Ok(new_connection.split()?)
        }
        Err(e) => {
            error!("Error while handling connection: {:?}", e);
            Err(Box::new(e.1))
        }
    }
}
