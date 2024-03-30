use log::error;
use nix::pty::forkpty;
use nix::unistd::execvp;
use std::ffi::CString;
use std::os::fd::{AsFd, AsRawFd};
use std::os::unix::io::OwnedFd;

pub struct Pty {
    pub master_fd: OwnedFd,
}

static DEFAULT_SHELL: &str = "bash";
const BUFFER_SIZE: usize = 1024;

impl Pty {
    pub fn new() -> Pty {
        unsafe {
            let pty_result = forkpty(None, None).unwrap();
            let master_fd = pty_result.master;
            let forked_process = pty_result.fork_result;

            if forked_process.is_child() {
                match execvp(
                    &CString::new(DEFAULT_SHELL).unwrap(),
                    &[CString::new(DEFAULT_SHELL).unwrap()],
                ) {
                    Ok(_) => (),
                    Err(e) => {
                        error!("Error executing shell: {:?}", e);
                        std::process::exit(1);
                    }
                }
            }

            #[allow(unreachable_code)]
            Pty { master_fd }
        }
    }

    pub fn listen(self, mut handle_pty_data: Box<dyn FnMut(&str)>) {
        let mut buf = [0u8; BUFFER_SIZE];
        let fd = self.master_fd.as_fd().as_raw_fd();
        loop {
            let read_result = match nix::unistd::read(fd, &mut buf) {
                Ok(n) => n,
                Err(e) => {
                    error!("Error reading from pty: {:?}", e);
                    break;
                }
            };

            match read_result {
                0 => break,
                n => {
                    let s = std::str::from_utf8(&buf[..n]).unwrap();
                    handle_pty_data(s);
                }
            }
        }
    }
}
