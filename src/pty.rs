use nix::pty::forkpty;
use nix::unistd::execvp;
use std::ffi::CString;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd};
use std::os::unix::io::OwnedFd;
use std::sync::{Arc, Mutex};


// pub fn write_to_pty(pty_fd: BorrowedFd, data: &str) {
//     match nix::unistd::write(pty_fd, data.as_bytes()) {
//         Ok(_) => println!("Write success"),
//         Err(e) => println!("Write error: {:?}", e)
//     }
// }

pub struct Pty {
    pub master_fd: OwnedFd,
    // handle_pty_data: Arc<Mutex<Box<dyn FnMut(&str)>>>
}

impl Pty {

    pub fn new() -> Pty{
        unsafe {
            let pty_result = forkpty(None, None).unwrap();
            let master_fd = pty_result.master;
            let forked_process = pty_result.fork_result;
            if forked_process.is_child() {
                
                execvp(&CString::new("bash").unwrap(), &[CString::new("bash").unwrap()]).unwrap();

                Pty {
                    master_fd,
                    // handle_pty_data:  Arc::from(Mutex::from(handle_pty_data))
                }

            } else {
                Pty {
                    master_fd,
                    // handle_pty_data: Arc::from(Mutex::from(handle_pty_data))
                }
                // let mut buf = [0u8; 1024];
                // let fd = master_fd.as_fd().as_raw_fd();
                // // let borrowed_fd = BorrowedFd::from(master_fd.as_fd());
                // loop {
                //     match nix::unistd::read(fd, &mut buf).unwrap(){
                //         0 => break,
                //         n => {
                //             let s = std::str::from_utf8(&buf[..n]).unwrap();
                //             handle_pty_data(s);
                //         }
                //     }
                // }
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

pub fn open_pty() {

    unsafe {
    let pty_result = forkpty(None, None).unwrap();
    let master_fd = pty_result.master;
    let forked_process = pty_result.fork_result;
    if forked_process.is_child() {
        
        execvp(&CString::new("bash").unwrap(), &[CString::new("bash").unwrap()]).unwrap();

    } else {
        let mut buf = [0u8; 1024];
        let fd = master_fd.as_fd().as_raw_fd();
        let borrowed_fd = BorrowedFd::from(master_fd.as_fd());

        // write_to_pty(master_fd, "echo Hello from pty\n");


        std::thread::sleep(std::time::Duration::from_secs(3));
        loop {

            match nix::unistd::read(fd, &mut buf){
                Ok(0) => break,
                Ok(n) => {
                    let s = std::str::from_utf8(&buf[..n]).unwrap();
                    print!("{}", s);
                },
                Err(e) => {
                    println!("Error: {:?}", e);
                    break;
                }

            }


        }
    }

    }

}
