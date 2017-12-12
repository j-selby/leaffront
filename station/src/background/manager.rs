/// Manages the loading of backgrounds in the ... background.

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

use std::fs::DirEntry;
use std::fs::read_dir;
use std::io::Result as IoResult;
use std::path::Path;
use std::error::Error;

use std::thread;
use std::thread::JoinHandle;

use rand::thread_rng;
use rand::Rng;

use image::open;
use image::DynamicImage;

enum BackgroundMessage {
    NextBackground,
    Exit
}

pub struct BackgroundManager {
    input   : Sender<BackgroundMessage>,
    output  : Receiver<DynamicImage>,
    handle  : Option<JoinHandle<()>>
}

impl BackgroundManager {
    pub fn next(&self) {
        self.input.send(BackgroundMessage::NextBackground).unwrap();
    }

    pub fn get_next(&self) -> Option<DynamicImage> {
        self.output.try_iter().next()
    }

    pub fn new(directory : String) -> Self {
        let (tx, rx): (Sender<BackgroundMessage>,
                       Receiver<BackgroundMessage>) = mpsc::channel();

        let (img_tx, img_rx): (Sender<DynamicImage>,
                               Receiver<DynamicImage>) = mpsc::channel();

        let handle = thread::spawn(move || {
            let directory = directory;
            let directory = Path::new(&directory);

            let mut rng = thread_rng();

            loop {
                match rx.recv().unwrap() {
                    BackgroundMessage::NextBackground => {},
                    BackgroundMessage::Exit => break
                }

                if !directory.exists() {
                    println!("art directory does not exist!");
                    continue;
                }

                let files = read_dir(directory).unwrap();
                let files: Vec<IoResult<DirEntry>> = files.collect();

                let mut new_files = Vec::new();

                for file in files {
                    match file {
                        Ok(value) => {
                            new_files.push(value)
                        },
                        Err(_) => {},
                    }
                }

                if new_files.len() <= 0 {
                    println!("No files found in art directory!");
                    continue;
                }

                let mut itr = 0;
                let bg_img = 'file: loop {
                    itr += 1;

                    // Randomly select file
                    let file = rng.gen_range(0, new_files.len());

                    let file : &DirEntry = &new_files[file];

                    // TODO: Use hardware decoder?
                    let bg_img = open(file.path());
                    let bg_img = match bg_img {
                        Ok(msg) => msg,
                        Err(msg) => {
                            println!("Error while loading image {}: {}", file.path().display(), msg.description());

                            if itr > 10 {
                                panic!("Failed to find a valid image after 10 attempts.");
                            } else {
                                continue 'file
                            }
                        },
                    };

                    break 'file bg_img;
                };

                img_tx.send(bg_img).unwrap();
            }
        });

        BackgroundManager {
            input : tx,
            output : img_rx,
            handle : Some(handle)
        }
    }
}

impl Drop for BackgroundManager {
    fn drop(&mut self) {
        self.input.send(BackgroundMessage::Exit).unwrap();
        self.handle.take().unwrap().join().unwrap();
    }
}