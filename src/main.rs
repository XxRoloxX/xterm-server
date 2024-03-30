mod pty;
mod ws;
use log::error;

fn main() {
    env_logger::init();
    match ws::run_xterm_server() {
        Ok(_) => (),
        Err(e) => {
            error!("Error while running sync_websockets: {:?}", e);
        }
    }
}
