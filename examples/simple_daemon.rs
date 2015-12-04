extern crate unix_daemonize;

use std::{io, env, time, thread, process};
use std::io::Write;
use unix_daemonize::daemonize_log;

fn main() {
    let mut args = env::args();
    let cmd_proc = args.next().unwrap();
    if let (Some(stdout_filename), Some(stderr_filename)) = (args.next(), args.next()) {
        println!("Ready to daemonize, target stdout_filename = {}, stderr_filename = {}", stdout_filename, stderr_filename);
        daemonize_log(stdout_filename, stderr_filename).unwrap();
        for _ in 0 .. 10 {
            println!("A string for stdout!");
            writeln!(&mut io::stdout(), "Another string for stdout!").unwrap();
            writeln!(&mut io::stderr(), "A string for stderr!").unwrap();
            thread::sleep(time::Duration::from_millis(1000));
        }
        println!("Successfull termination");
    } else {
        writeln!(&mut io::stderr(), "Usage: {} <stdout_filename> <stderr_filename>", cmd_proc).unwrap();
        process::exit(1);
    }
}
