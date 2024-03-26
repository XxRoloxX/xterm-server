mod pty;
mod ws;
#[tokio::main]
async fn main() {
    // println!("Hello, world!");
    // pty::open_pty();
    ws::start_ws().await;
}
