extern crate libc;

use std::{io, env, ffi, process};
use std::path::Path;

#[derive(Debug)]
pub enum Error {
    FirstFork(Option<i32>),
    SecondFork(Option<i32>),
    Setsid(Option<i32>),
    Chdir(io::Error),
    StdoutFilenameToStr,
    StderrFilenameToStr,
    StdoutFilenameFFI(ffi::NulError),
    StderrFilenameFFI(ffi::NulError),
    OpenStdout(Option<i32>),
    OpenStderr(Option<i32>),
}

macro_rules! errno {
    ($err:ident) => ({ Error::$err(io::Error::last_os_error().raw_os_error()) })
}

pub fn daemonize_redirect<PO, PE>(stdout: Option<PO>, stderr: Option<PE>) -> Result<libc::pid_t, Error>
    where PO: AsRef<Path>, PE: AsRef<Path>
{
    let stdout_filename = stdout.as_ref()
        .map(|s| s.as_ref().to_str())
        .unwrap_or(Some("/dev/null"))
        .ok_or(Error::StdoutFilenameToStr);
    let stdout_path = try!(ffi::CString::new(try!(stdout_filename)).map_err(|e| Error::StdoutFilenameFFI(e)));

    let stderr_filename = stderr.as_ref()
        .map(|s| s.as_ref().to_str())
        .unwrap_or(Some("/dev/null"))
        .ok_or(Error::StderrFilenameToStr);
    let stderr_path = try!(ffi::CString::new(try!(stderr_filename)).map_err(|e| Error::StderrFilenameFFI(e)));

    let stdout_fd = unsafe { libc::open(stdout_path.as_ptr(),
                                        libc::O_CREAT | libc::O_WRONLY | libc::O_APPEND,
                                        (libc::S_IRUSR | libc::S_IRGRP | libc::S_IWGRP | libc::S_IWUSR) as libc::c_uint) };
    if stdout_fd < 0 {
        return Err(errno!(OpenStdout))
    }

    let stderr_fd = unsafe { libc::open(stderr_path.as_ptr(),
                                        libc::O_CREAT | libc::O_WRONLY | libc::O_APPEND,
                                        (libc::S_IRUSR | libc::S_IRGRP | libc::S_IWGRP | libc::S_IWUSR) as libc::c_uint) };
    if stderr_fd < 0 {
        unsafe { libc::close(stdout_fd) };
        return Err(errno!(OpenStderr))
    }

    daemonize(stdout_fd, stderr_fd)
}

fn daemonize(stdout_fd: libc::c_int, stderr_fd: libc::c_int) -> Result<libc::pid_t, Error> {
    let pid = unsafe { libc::fork() };
    if pid == -1 {
        unsafe { libc::close(stdout_fd) };
        unsafe { libc::close(stderr_fd) };
        return Err(errno!(FirstFork))
    } else if pid != 0 {
        process::exit(0)
    }

    if unsafe { libc::setsid() } == -1 {
        unsafe { libc::close(stdout_fd) };
        unsafe { libc::close(stderr_fd) };
        return Err(errno!(Setsid))
    }

    unsafe { libc::signal(libc::SIGHUP, libc::SIG_IGN); }
    let pid = unsafe { libc::fork() };
    if pid == -1 {
        unsafe { libc::close(stdout_fd) };
        unsafe { libc::close(stderr_fd) };
        return Err(errno!(SecondFork))
    } else if pid != 0 {
        process::exit(0)
    }

    match env::set_current_dir("/") {
        Ok(()) => (),
        Err(e) => {
            unsafe { libc::close(stdout_fd) };
            unsafe { libc::close(stderr_fd) };
            return Err(Error::Chdir(e))
        },
    }

    unsafe {
        libc::dup2(stdout_fd, libc::STDOUT_FILENO);
        libc::dup2(stderr_fd, libc::STDERR_FILENO);
    }

    Ok(pid)
}
