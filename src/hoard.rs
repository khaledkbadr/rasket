use std::borrow::BorrowMut;
use std::io::BufReader;
use std::path::PathBuf;

use futures::future;
use futures::sync::mpsc;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;
use tokio::fs::file as tokio_file;
use tokio::io;
use tokio::prelude::*;

// struct LogFile {
//     handler: tokio_file::OpenFuture<PathBuf>,
//     path: PathBuf,
// }

// impl LogFile {
//     fn new(file: PathBuf) -> Self {
//         LogFile {
//             handler: tokio_file::File::open(file.copy),
//             path: file,
//         }
//     }
// }

/// Manages the operations done on the log files being watched
#[derive(Debug)]
struct LogFiles {
    /// Holds the log files reader
    readers: Vec<BufReader<tokio_file::File>>,
}

impl LogFiles {
    /// Construct LogFile from vector of paths provided
    /// opens the given files then wraps it with BufReader
    fn from(files: Vec<PathBuf>) -> Self {
        let mut readers: Vec<BufReader<tokio_file::File>> = vec![];
        let (tx, mut rx) = mpsc::unbounded();

        let path_stream = stream::iter_ok(files);

        let fut = path_stream.for_each(move |file| {
            let cloned_readers = tx.clone();
            let open_file = tokio_file::File::open(file)
                .and_then(move |fd| {
                    let reader = BufReader::new(fd);
                    cloned_readers
                        .unbounded_send(reader)
                        .expect("Failed to send reader");
                    Ok(())
                })
                .map_err(|e| panic!("Open file error: {:?}", e));
            tokio::spawn(open_file);
            Ok(())
        });

        tokio::run(fut);

        while let Ok(Async::Ready(val)) = rx.poll() {
            match val {
                Some(val) => readers.push(val),
                None => break,
            }
        }

        LogFiles { readers }
    }
}

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Activate debug mode
    #[structopt(short = "d", long = "debug")]
    debug: bool,

    /// Activate verbose mode
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

// /// Log extractor
// /// read and
// fn extractor(files: Vec<PathBuf>) {
//     let log_files = LogFiles::from(files);
// }

fn main() {
    let mut opt = Opt::from_args();
    opt.files.dedup();

    let log_files = LogFiles::from(opt.files);
    println!("{:?}", log_files);
}
