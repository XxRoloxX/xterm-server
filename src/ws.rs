use websocket::sync::Server;
use websocket::Message;
use std::thread;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

pub fn sync_websockets(){
    let mut ws_server = Server::bind("127.0.0.1:3030").unwrap();
    let (tx, rx) = mpsc::channel::<String>();
    let tx1 = tx.clone();
    let mpsc_receiver = Arc::from(Mutex::from(rx));
        
    thread::spawn(move || {
        loop {
            tx1.send("Hello from server!".to_string()).unwrap();
            thread::sleep(std::time::Duration::from_secs(1));
        }
    });


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
            }
        });


    
    }
}

// pub async fn start_ws() {
//     let routes = warp::path("echo").and(warp::ws()).map(|ws: warp::ws::Ws| {
//         ws.on_upgrade(|websocket| async {
//             let (mut tx, mut rx) = websocket.split();
//             let (mtx, mrx) = mpsc::channel::<String>();
//             // let tx_clone = Arc::from(Mutex::from(tx));
//             // let tx_clone_2 = tx_clone.clone();
//
//             // non asynchronous Channel for sharing messages between threads
//             // let mtx_2 = mt.clone();
//             // let mtx_clone = Arc::from(Mutex::from(mtx));
//             // let mtx_2 = mtx_clone.clone();
//
//             // thread::spawn(|| async move {
//             //     let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
//             //     loop {
//             //         interval.tick().await;
//             //         tx.send(Message::text("Hello from server!")).await.unwrap();
//             //     }
//             // });
//
//             let handle = thread::spawn(|| async move {
//                 loop {
//                     match mrx.recv() {
//                         Ok(i) => {
//                             println!("Received message from mpsc: {}", i);
//                             tx.send(Message::text(i)).await.unwrap();
//                             // println!("Received message: {}", i);
//                         }
//                         Err(e) => {
//                             eprintln!("Error receiving message from mpsc: {}", e);
//                             break;
//                         } // println!("Received message: {}", i);
//                     }
//                 }
//                 // while let Ok(msg) = mrx.recv() {
//                 //     tx.send(Message::text(msg)).await.unwrap();
//                 // }
//             });
//
//             let handle2 = thread::spawn(|| async move {
//                 while let Some(result) = rx.next().await {
//                     let msg = match result {
//                         Ok(msg) => {
//                             let msg = String::from("From server: ") + msg.to_str().unwrap();
//                             println!("Received message: {}", msg);
//                             Message::text(msg)
//                         }
//                         Err(e) => {
//                             eprintln!("websocket error: {:?}", e);
//                             break;
//                         }
//                     };
//
//                     match mtx.send(msg.to_str().unwrap().into()) {
//                         Ok(_) => {
//                             println!("Message sent to channel");
//                         }
//                         Err(e) => {
//                             eprintln!("Error sending message to channel: {}", e);
//                         }
//                     }
//                 }
//             });
//
//             handle2.join().unwrap().await;
//             handle.join().unwrap().await;
//         })
//     });
//     warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
// }
