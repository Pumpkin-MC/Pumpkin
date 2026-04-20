use libc::EIO;
use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jint, jstring};
use nix::poll::{PollFd, PollFlags, poll};
use nix::pty::openpty;
use nix::unistd::{dup2, setsid};
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, BorrowedFd, FromRawFd, IntoRawFd};
use std::os::unix::process::CommandExt;
use std::process::Command;

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_pumpkinmc_MainActivity_openPty(
    mut env: JNIEnv,
    _class: JClass,
    command: JString,
    cwd: JString,
) -> jint {
    let cmd_str: String = env.get_string(&command).unwrap().into();
    let working_dir: String = env.get_string(&cwd).unwrap().into();

    let pty = openpty(None, None).expect("openpty failed");
    let master_fd = pty.master;
    let slave_fd = pty.slave;

    unsafe {
        match nix::unistd::fork() {
            Ok(nix::unistd::ForkResult::Child) => {
                let _ = setsid();
                let slave_raw = slave_fd.as_raw_fd();
                let _ = dup2(slave_raw, 0);
                let _ = dup2(slave_raw, 1);
                let _ = dup2(slave_raw, 2);
                drop(slave_fd);
                drop(master_fd);

                let _ = Command::new(&cmd_str).current_dir(working_dir).exec();
                std::process::exit(1);
            }
            Ok(nix::unistd::ForkResult::Parent { child: _ }) => {
                drop(slave_fd);
                master_fd.into_raw_fd() as jint
            }
            Err(_) => -1,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_pumpkinmc_MainActivity_writePty(
    mut env: JNIEnv,
    _class: JClass,
    fd: jint,
    text: JString,
) {
    let input: String = env.get_string(&text).unwrap().into();
    let mut file = unsafe { File::from_raw_fd(fd) };
    let _ = file.write_all(input.as_bytes());
    let _ = file.flush();
    let _ = file.into_raw_fd();
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_pumpkinmc_MainActivity_readPty(
    env: JNIEnv,
    _class: JClass,
    fd: jint,
) -> jstring {
    let borrowed = unsafe { BorrowedFd::borrow_raw(fd) };
    let mut pfd = [PollFd::new(&borrowed, PollFlags::POLLIN)];
    if poll(&mut pfd, 0).unwrap_or(0) == 0 {
        return std::ptr::null_mut();
    }

    let mut buffer = [0u8; 4096];
    let mut file = unsafe { File::from_raw_fd(fd) };
    let result = match file.read(&mut buffer) {
        Ok(n) if n > 0 => {
            let s = String::from_utf8_lossy(&buffer[..n]);
            Some(env.new_string(s).unwrap().into_raw())
        }
        Ok(_) => {
            let _ = file.into_raw_fd();
            return env.new_string("\n[Server stopped]\n").unwrap().into_raw();
        }
        Err(e) if e.raw_os_error() == Some(EIO) => {
            let _ = file.into_raw_fd();
            return env.new_string("\n[Server stopped]\n").unwrap().into_raw();
        }
        _ => None,
    };
    let _ = file.into_raw_fd();
    result.unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_pumpkinmc_MainActivity_closePty(
    _env: JNIEnv,
    _class: JClass,
    fd: jint,
) {
    let _ = nix::unistd::close(fd);
}
