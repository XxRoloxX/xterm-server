use websocket::sync::Server;
use websocket::Message;
use std::thread;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use super::pty::Pty;
use websocket::OwnedMessage;

pub fn sync_websockets(){
    let mut ws_server = Server::bind("127.0.0.1:3030").unwrap();
    let (tx, rx) = mpsc::channel::<String>();
    let mpsc_receiver = Arc::from(Mutex::from(rx));

    loop {
        let result = match ws_server.accept() {
            Ok(ws_upgrade) => {
                println!("Connection accepted");
                ws_upgrade
            },
            Err(e) => {
                eprintln!("Error accepting connection: {:?}", e);
                continue;
            }
        };

        let (mut receiver,mut sender ) = result.accept().unwrap().split().unwrap();
        let mpsc_rc_clone = mpsc_receiver.clone();
        let tx1_clone = tx.clone();
        let pty = Pty::new();
        let pty_fd = pty.master_fd.try_clone().unwrap();

        thread::spawn(move || {

            let handle_pty_data = Box::new(move |data: &str| {
                tx1_clone.send(data.to_string()).unwrap();
            });

            pty.listen(handle_pty_data);
        });


        thread::spawn(move || {
            loop {
                match mpsc_rc_clone.lock().unwrap().recv() {
                    Ok(msg) => {
                        println!("Received message from channel: {:?}", msg);
                        sender.send_message(&Message::text(msg)).unwrap();
                    },
                    Err(e) => {
                        eprintln!("Error receiving message from channel: {:?}", e);
                        break;
                    }
                }
            }
        
        });

        thread::spawn(move || {
            while let Ok(msg) = receiver.recv_message() {
                println!("Received message: {:?}", msg);
                match msg {
                    OwnedMessage::Text(msg) => {
                        nix::unistd::write(&pty_fd, msg.as_bytes());
                    },
                    OwnedMessage::Binary(msg) => {
                        nix::unistd::write(&pty_fd, &msg);
                    }
                    _ => {
                        println!("Received non-text message");
                    }
                }
            }
        });
    }
}
