#![feature(collections,exit_status,io)]

extern crate getopts;
extern crate tempdir;

use getopts::Options;
use std::fmt;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;


#[allow(dead_code)]
fn usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] <what> <where>", program);
    print!("{}", opts.usage(&brief[..]));
    env::set_exit_status(3);
}


#[derive(Debug)]
pub enum GrepError {
    Read(String),
    File(String)
}


impl fmt::Display for GrepError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GrepError::Read(ref s) => write!(f, "{}", s),
            GrepError::File(ref s) => write!(f, "{}", s)
        }
    }
}


pub fn grep(what: String, file: String, icase: bool) -> Result<bool, GrepError> {
    let path = Path::new(&*file);
    let iwhat = match icase {
        true => what.to_lowercase(),
        false => what
    };
    let mut stream = if file == "-" {
        Box::new(io::stdin()) as Box<Read>
    } else {
        match File::open(&path) {
            Ok(fstream) => Box::new(fstream) as Box<Read>,
            Err(e) => {
                let gerror = GrepError::File(format!("couldn't open {}: {}", file, e));
                return Err(gerror);
            }
        }
    };

    // TODO: mmap()
    let mut content = String::new();
    match stream.read_to_string(&mut content) {
        Ok(_) => {},
        Err(e) => {
            let gerror = GrepError::Read(format!("couldn't read: {:?}", e));
            return Err(gerror);
        }
    }

    // TODO: don't create lines[]
    let mut found = false;
    let lines = content.split("\n");
    for line in lines {
        match line.find(&iwhat[..]) {
            Some(_) => {
                println!("{}", line);
                found = true;
            }
            _ => {}
        }
    }
    Ok(found)
}


#[allow(dead_code)]
fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();

    opts.optflag("i", "ignore-case", "ignore case when matching");
    opts.optflag("h", "help", "print this help menu");

    let parsed = match opts.parse(args.tail()) {
        Ok(p) => p,
        Err(_) => {
            usage(&program[..], opts);
            return;
        }
    };

    if parsed.opt_present("h") {
        usage(&program[..], opts);
        return;
    }

    let icase = parsed.opt_present("ignore-case");

    let (what, file) = if parsed.free.len() == 1 {
        (parsed.free[0].clone(), "-".to_string())
    } else if parsed.free.len() == 2 {
        (parsed.free[0].clone(), parsed.free[1].clone())
    } else {
        usage(&program[..], opts);
        return;
    };

    let retval = match grep(what, file, icase) {
        Ok(found) => {
            if found == true {
                0
            } else {
                1
            }
        },
        Err(e) => {
            println!("{}", e);
            2
        }
    };

    env::set_exit_status(retval);
}


#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::fs::File;
    use tempdir::TempDir;

    use super::*;

    fn create_tempfile(dir: &str, file: &str, content: &[u8]) -> (TempDir, File, String) {
        let tempdir = TempDir::new(dir).ok().expect("couldn't create temp dir");
        let path = tempdir.path().join(file);
        let mut tempfile = File::create(&path).ok().expect("couldn't create temp file");
        let _ = tempfile.write_all(content);
        let _ = tempfile.flush();

        (tempdir, tempfile, format!("{}", path.into_os_string().into_string().unwrap()))
    }

    #[test]
    fn test_basic() {
        let (_tempdir, _tempfile, path) = create_tempfile("grep", "tmp", b"aaa\nbbb\nccc\n");
        let res = grep("bbb".to_string(), path, false);
        assert!(res.is_ok() && res.unwrap() == true);
    }

    #[test]
    fn test_ignore_case() {
        let (_tempdir, _tempfile, path) = create_tempfile("grep", "tmp", b"aaa\nbbb\nccc\n");
        let res = grep("bbb".to_string(), path, true);
        assert!(res.is_ok() && res.unwrap() == true);
    }
}
