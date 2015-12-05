extern crate libc;

use std::{io, env, ffi, path, process};

#[derive(Debug)]
pub enum Error {
    FirstFork(Option<i32>),
    SecondFork(Option<i32>),
    Setsid(Option<i32>),
    Chdir(io::Error),
    FilenameToStr(path::PathBuf),
    FilenameFFI(path::PathBuf, ffi::NulError),
    OpenStd(path::PathBuf, Option<i32>),
    Dup2(Option<i32>),
}

#[derive(Debug)]
pub enum ChdirMode {
    NoChdir,
    ChdirRoot,
}

struct Redirected(libc::c_int);

impl Drop for Redirected {
    fn drop(&mut self) {
        if self.0 >= 0 {
            unsafe { libc::close(self.0) };
            self.0 = -1;
        }
    }
}

fn to_path_buf<P>(path: &Option<P>) -> path::PathBuf where P: AsRef<path::Path> {
    if let &Some(ref p) = path {
        p.as_ref().to_owned()
    } else {
        let null: &path::Path = "/dev/null".as_ref();
        null.to_path_buf()
    }
}

fn redirect<P>(std: Option<P>) -> Result<Redirected, Error> where P: AsRef<path::Path> {
    let filename = std.as_ref()
        .map(|s| s.as_ref().to_str())
        .unwrap_or(Some("/dev/null"))
        .ok_or(Error::FilenameToStr(to_path_buf(&std)));
    let path = try!(ffi::CString::new(try!(filename)).map_err(|e| Error::FilenameFFI(to_path_buf(&std), e)));

    let fd = unsafe { libc::open(path.as_ptr(),
                                 libc::O_CREAT | libc::O_WRONLY | libc::O_APPEND,
                                 (libc::S_IRUSR | libc::S_IRGRP | libc::S_IWGRP | libc::S_IWUSR) as libc::c_uint) };
    if fd < 0 {
        Err(Error::OpenStd(to_path_buf(&std), io::Error::last_os_error().raw_os_error()))
    } else {
        Ok(Redirected(fd))
    }
}

pub fn daemonize_redirect<PO, PE>(stdout: Option<PO>, stderr: Option<PE>, chdir: ChdirMode) -> Result<libc::pid_t, Error>
    where PO: AsRef<path::Path>, PE: AsRef<path::Path>
{
    daemonize(try!(redirect(stdout)), try!(redirect(stderr)), chdir)
}

fn daemonize(mut stdout_fd: Redirected, mut stderr_fd: Redirected, chdir: ChdirMode) -> Result<libc::pid_t, Error> {
    macro_rules! errno {
        ($err:ident) => ({ return Err(Error::$err(io::Error::last_os_error().raw_os_error())) })
    }

    let pid = unsafe { libc::fork() };
    if pid < 0 {
        errno!(FirstFork)
    } else if pid != 0 {
        process::exit(0)
    }

    if unsafe { libc::setsid() } < 0 {
        errno!(Setsid)
    }

    unsafe { libc::signal(libc::SIGHUP, libc::SIG_IGN); }
    let pid = unsafe { libc::fork() };
    if pid < 0 {
        errno!(SecondFork)
    } else if pid != 0 {
        process::exit(0)
    }

    if let ChdirMode::ChdirRoot = chdir {
        match env::set_current_dir("/") {
            Ok(()) => (),
            Err(e) => {
                return Err(Error::Chdir(e))
            },
        }
    }

    if unsafe { libc::dup2(stdout_fd.0, libc::STDOUT_FILENO) } < 0 {
        errno!(Dup2)
    } else {
        stdout_fd.0 = -1
    }

    if unsafe { libc::dup2(stderr_fd.0, libc::STDERR_FILENO) } < 0 {
        errno!(Dup2)
    } else {
        stderr_fd.0 = -1
    }

    Ok(pid)
}
