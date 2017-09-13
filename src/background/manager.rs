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

use videocore::dispmanx;
use videocore::dispmanx::ElementHandle;
use videocore::dispmanx::ResourceHandle;
use videocore::dispmanx::Transform;
use videocore::image::ImageType;
use videocore::image::Rect as VCRect;

use rand::thread_rng;
use rand::Rng;

use image::open;
use image::RgbImage;

use libc::c_void;

enum BackgroundMessage {
    NextBackground,
    Exit
}

pub struct BackgroundManager {
    input   : Sender<BackgroundMessage>,
    handle  : Option<JoinHandle<()>>
}

impl BackgroundManager {
    pub fn next(&self) {
        self.input.send(BackgroundMessage::NextBackground).unwrap();
    }

    pub fn new(directory : String, element : ElementHandle) -> Self {
        let (tx, rx): (Sender<BackgroundMessage>,
                       Receiver<BackgroundMessage>) = mpsc::channel();

        let handle = thread::spawn(move || {
            let directory = directory;
            let directory = Path::new(&directory);

            let mut old_resource : Option<ResourceHandle> = None;

            let mut rng = thread_rng();

            loop {
                match rx.recv().unwrap() {
                    BackgroundMessage::NextBackground => {
                        match old_resource {
                            Some(resource) => {
                                dispmanx::resource_delete(resource);
                            },
                            _ => {}
                        }
                    },
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
                let bg_img : RgbImage = 'file: loop {
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

                    break 'file bg_img.to_rgb();
                };

                // Resize the background to the correct size
                //let size = Context::get_resolution();

                // Pad out the image, if required
                let target_width;
                let target_height;
                let padding;

                let mut img_buffer = if bg_img.width() % 16 != 0 {
                    // Find the next multiple that *is* even
                    padding = 16 - (bg_img.width() % 16);
                    target_width = bg_img.width();
                    target_height = bg_img.height();

                    let old_width = bg_img.width();

                    let bg_img = bg_img.to_vec();

                    let mut buf: Vec<u8> = vec![0; ((target_width + padding) * target_height * 3) as usize];
                    for y in 0 .. target_height {
                        for x in 0 .. old_width {
                            buf[((y * (target_width + padding) + x) * 3) as usize]
                                = bg_img[((y * old_width + x) * 3) as usize];
                            buf[((y * (target_width + padding) + x) * 3 + 1) as usize]
                                = bg_img[((y * old_width + x) * 3 + 1) as usize];
                            buf[((y * (target_width + padding) + x) * 3 + 2) as usize]
                                = bg_img[((y * old_width + x) * 3 + 2) as usize];
                        }
                    }

                    buf
                } else {
                    target_width = bg_img.width();
                    target_height = bg_img.height();
                    padding = 0;
                    bg_img.to_vec()
                };

                let bg_ptr = img_buffer.as_mut_ptr() as *mut _ as *mut c_void;
                let mut ptr = 0; // Unused

                let dest_rect = VCRect {
                    x: 0,
                    y: 0,
                    width: target_width as i32,
                    height: target_height as i32
                };

                let bg_resource = dispmanx::resource_create(ImageType::RGB888,
                                                            target_width as u32,
                                                            target_height as u32,
                                                            &mut ptr);

                if dispmanx::resource_write_data(bg_resource, ImageType::RGB888,
                                                 (3 * (target_width + padding)) as i32,
                                                 bg_ptr, &dest_rect) {
                    println!("Failed to write data")
                }

                let update = dispmanx::update_start(10);

                // Resize the element's src attr
                //println!("Img dims: {} {}", target_width, target_height);

                let src_rect = VCRect {
                    x : 0,
                    y : 0,
                    width : (target_width as i32) << 16,
                    height : (target_height as i32) << 16
                };

                dispmanx::element_change_attributes(update, element,
                                                    (1 << 3) | (1 << 2),
                                                    0, // Ignored
                                                    255, // Ignored
                                                    &src_rect,//&dest_rect,
                                                    &dest_rect,//&src_rect,
                                                    0, // Ignored
                                                    Transform::NO_ROTATE // Ignored
                );

                if dispmanx::element_change_source(update, element, bg_resource) {
                    println!("Resource change failed!");
                }

                if dispmanx::update_submit_sync(update) {
                    println!("Failed to update");
                }

                old_resource = Some(bg_resource);
            }

            print!("Cleaning up background: ");

            match old_resource {
                Some(resource) => {
                    dispmanx::resource_delete(resource);
                },
                _ => {}
            }
            println!("Done!");

        });

        BackgroundManager {
            input : tx,
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