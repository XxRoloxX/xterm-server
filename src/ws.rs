use futures::{SinkExt, StreamExt};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use warp::ws::Message;
use warp::Filter;

pub async fn start_ws() {
    let routes = warp::path("echo").and(warp::ws()).map(|ws: warp::ws::Ws| {
        ws.on_upgrade(|websocket| async {
            let (mut tx, mut rx) = websocket.split();
            let (mtx, mrx) = mpsc::channel::<String>();
            // let tx_clone = Arc::from(Mutex::from(tx));
            // let tx_clone_2 = tx_clone.clone();

            // non asynchronous Channel for sharing messages between threads
            // let mtx_2 = mt.clone();
            // let mtx_clone = Arc::from(Mutex::from(mtx));
            // let mtx_2 = mtx_clone.clone();

            // thread::spawn(|| async move {
            //     let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            //     loop {
            //         interval.tick().await;
            //         tx.send(Message::text("Hello from server!")).await.unwrap();
            //     }
            // });

            let handle = thread::spawn(|| async move {
                loop {
                    match mrx.recv() {
                        Ok(i) => {
                            println!("Received message from mpsc: {}", i);
                            tx.send(Message::text(i)).await.unwrap();
                            // println!("Received message: {}", i);
                        }
                        Err(e) => {
                            eprintln!("Error receiving message from mpsc: {}", e);
                            break;
                        } // println!("Received message: {}", i);
                    }
                }
                // while let Ok(msg) = mrx.recv() {
                //     tx.send(Message::text(msg)).await.unwrap();
                // }
            });

            let handle2 = thread::spawn(|| async move {
                while let Some(result) = rx.next().await {
                    let msg = match result {
                        Ok(msg) => {
                            let msg = String::from("From server: ") + msg.to_str().unwrap();
                            println!("Received message: {}", msg);
                            Message::text(msg)
                        }
                        Err(e) => {
                            eprintln!("websocket error: {:?}", e);
                            break;
                        }
                    };

                    match mtx.send(msg.to_str().unwrap().into()) {
                        Ok(_) => {
                            println!("Message sent to channel");
                        }
                        Err(e) => {
                            eprintln!("Error sending message to channel: {}", e);
                        }
                    }
                }
            });
            handle.join().unwrap().await;
            handle2.join().unwrap().await;
        })
    });
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
