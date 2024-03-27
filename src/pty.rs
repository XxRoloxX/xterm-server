use nix::pty::forkpty;
use nix::unistd::execvp;
use std::ffi::CString;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd};
use std::os::unix::io::OwnedFd;


pub struct Pty {
    pub master_fd: OwnedFd,
}

impl Pty {

    pub fn new() -> Pty{
        unsafe {
            let pty_result = forkpty(None, None).unwrap();
            let master_fd = pty_result.master;
            let forked_process = pty_result.fork_result;

            if forked_process.is_child() {
                execvp(&CString::new("bash").unwrap(), &[CString::new("bash").unwrap()]).unwrap();
            }         

            Pty {
                master_fd
            }
        }
    }

    pub fn listen(self,mut handle_pty_data: Box<dyn FnMut(&str)>) {
        let mut buf = [0u8; 1024];
        let fd = self.master_fd.as_fd().as_raw_fd();
        loop {
            match nix::unistd::read(fd, &mut buf).unwrap(){
                0 => break,
                n => {
                    let s = std::str::from_utf8(&buf[..n]).unwrap();
                    handle_pty_data(s);
                }
            }
        }
    }
}

