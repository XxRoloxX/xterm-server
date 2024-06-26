use super::pty::Pty;
use log::{error, info};
use std::env;
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

const DEFAULT_BIND_PORT: &str = "8080";
const BIND_ADDRESS: &str = "0.0.0.0:";
const XTERM_PORT_ENV: &str = "XTERM_PORT";

pub fn run_xterm_server() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = bind_to_ws_socket()?;

    loop {
        // The main thread will wait for a connection to be established
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

        // Pty master fd is used to both read and write to the pty (it is connected to the stdin/stdout of the forked process)
        let pty_fd = match pty.master_fd.try_clone() {
            Ok(fd) => fd,
            Err(e) => {
                error!("Error while cloning pty master fd: {:?}", e);
                continue;
            }
        };

        // When the pty outputs on the master fd, it will send the data to the mpsc channel
        let _handle_pty_output_thread = handle_pty_output(shared_tx, pty);

        // The mpsc data thread will listen for data on the mpsc channel and send it to the websocket
        let _handle_mpsc_data_thread = handle_mpsc_data(shared_rx.clone(), sender);

        // The websocket message thread will listen for messages from the websocket and write them to the pty
        let _handle_websocket_message_thread = handle_websocket_message(receiver, pty_fd);
    }
}

pub fn bind_to_ws_socket() -> Result<WsServer<NoTlsAcceptor>, Box<dyn std::error::Error>> {
    let websocket_bind_addr = BIND_ADDRESS.to_string()
        + env::var(XTERM_PORT_ENV)
            .unwrap_or(DEFAULT_BIND_PORT.to_string())
            .as_str();

    let server = Server::bind(&websocket_bind_addr).map_err(|e| {
        error!("Couldn't bind to a socket {}:{}", websocket_bind_addr, e);
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
    rx: Arc<Mutex<mpsc::Receiver<String>>>,
    mut ws_sender: websocket::sender::Writer<std::net::TcpStream>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || loop {
        let receiver = rx.lock();

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
