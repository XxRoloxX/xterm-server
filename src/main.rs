mod pty;
mod ws;
fn main() {
    // println!("Hello, world!");
    // pty::open_pty();
    ws::sync_websockets();
}
