#![feature(set_stdio)]
extern crate libc;

use std::{io, fs, env, process};
use std::path::Path;
use libc::{fork, setsid};
use libc::{signal, SIGHUP, SIG_IGN};

#[derive(Debug)]
pub enum Error {
    FirstFork(Option<i32>),
    SecondFork(Option<i32>),
    Setsid(Option<i32>),
    Chdir(io::Error),
    OpenStdoutLog(io::Error),
    OpenStderrLog(io::Error),
}

macro_rules! errno {
    ($err:ident) => ({ Error::$err(io::Error::last_os_error().raw_os_error()) })
}

pub fn daemonize_log<PO, PE>(stdout_log: PO, stderr_log: PE) -> Result<(), Error> where PO: AsRef<Path>, PE: AsRef<Path> {
    let stdout = try!(fs::File::create(stdout_log).map_err(|e| Error::OpenStdoutLog(e)));
    let stderr = try!(fs::File::create(stderr_log).map_err(|e| Error::OpenStderrLog(e)));
    daemonize(io::BufWriter::new(stdout), io::BufWriter::new(stderr))
}

pub fn daemonize<WO, WE>(stdout_redirect: WO, stderr_redirect: WE) -> Result<(), Error>
    where WO: io::Write + Send + 'static, WE: io::Write + Send + 'static
{
    let pid = unsafe { fork() };
    if pid == -1 {
        return Err(errno!(FirstFork))
    } else if pid != 0 {
        process::exit(0)
    }

    if unsafe { setsid() } == -1 {
        return Err(errno!(Setsid))
    }

    unsafe { signal(SIGHUP, SIG_IGN); }
    let pid = unsafe { fork() };
    if pid == -1 {
        return Err(errno!(SecondFork))
    } else if pid != 0 {
        process::exit(0)
    }

    try!(env::set_current_dir("/").map_err(|e| Error::Chdir(e)));

    let _ = io::set_print(Box::new(stdout_redirect));
    let _ = io::set_panic(Box::new(stderr_redirect));
    Ok(())
}
