use nix::pty::forkpty;
use nix::unistd::execvp;
use std::ffi::CString;
use std::fs::File;
use std::os::unix::io::FromRawFd;
use nix::unistd::write;
use nix::pty::PtyMaster;
use nix::unistd::dup2;
// Read trait
use std::io::{Read, Write};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd};


pub fn open_pty() {

    unsafe {
    let pty_result = forkpty(None, None).unwrap();
    let master_fd = pty_result.master;
    let forked_process = pty_result.fork_result;
    // let reader = PtyMaster({master_fd});

    // CREATE PTYMASTER
    if forked_process.is_child() {
        
        // dup2(master_fd.as_fd().as_raw_fd(), 0);
        // dup2(master_fd.as_fd().as_raw_fd(), 1);
        // dup2(master_fd.as_fd().as_raw_fd(), 2);
        // Child process
        println!("Child process");
        execvp(&CString::new("bash").unwrap(), &[CString::new("bash").unwrap()]).unwrap();

        // Output to master 


        // std::thread::sleep(std::time::Duration::from_secs(2));


    } else {
        // Parent process
        println!("Parent process");
        let mut buf = [0u8; 1024];
        let fd = master_fd.as_fd().as_raw_fd();
        // let mut file = File::from_raw_fd(fd);

        // Write ls -l command to fd
        let ls = "ls -l\n";

        match  nix::unistd::write(BorrowedFd::borrow_raw(fd), ls.as_bytes()) {
            Ok(_) => println!("Write success"),
            Err(e) => println!("Write error: {:?}", e)
        }

        std::thread::sleep(std::time::Duration::from_secs(3));
        // Wait for 2 secunnds
        loop {

            // Read the fd status

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

    // Run command in slave


    // println!("master: {:?}, slave: {:?}", master, slave);
}
