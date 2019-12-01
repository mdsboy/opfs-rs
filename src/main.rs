use libc;
use std::env;
use std::ffi::CString;
use std::fs::File;
use std::os::unix::io::{AsRawFd, FromRawFd};
use memmap::MmapOptions;

use std::io::BufReader;
use std::io::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    if args.len() < 3 {
        println!("error");
        return;
    }

    let progname = &args[0];
    let img_file = &args[1];
    let cmd = &args[2];

    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(img_file)
        .unwrap();

    let img_size = unsafe {
        let root = CString::new(img_file.chars().map(|c| c as u8).collect::<Vec<u8>>()).unwrap();
        let mut stat: libc::stat = std::mem::zeroed();
        println!("{}", stat.st_blksize);
        if libc::stat(root.as_ptr(), &mut stat) >= 0 {
            println!("{}", stat.st_blksize);
        }
        stat.st_blksize as usize
    };
    println!("{}", img_size);
/*
    let img = unsafe {
        libc::mmap(
            ::std::ptr::null_mut(),
            img_size as usize,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            file.as_raw_fd(),
            0,
        )
    };
    println!("Address of mapped data: {:p}", img);
*/
    let img = unsafe {
        MmapOptions::new().map(&file).unwrap()
    };
    println!("{:?}", img);
/*
    let inner: Vec<u8> = (*img).to_vec();
    println!("{:?}", inner);
    println!("{}", inner.len());
*/
    //let ser = bincode::deserialize(&img).unwrap();
    /*let sblk = unsafe {
        std::mem::transmute::<std::ffi::c_void, superblock>(img.as_ptr())
    };*/
    
    println!("{}", (*img).first().unwrap());
    let bsize = 1024;
    let superblock = SuperBlock{
        magic: u32::from_be_bytes([(*img)[bsize+3], (*img)[bsize+2], (*img)[bsize+1], (*img)[bsize+0]]),
        size: u32::from_be_bytes([(*img)[bsize+7], (*img)[bsize+6], (*img)[bsize+5], (*img)[bsize+4]]),
        nblocks: u32::from_be_bytes([(*img)[bsize+11], (*img)[bsize+10], (*img)[bsize+9], (*img)[bsize+8]]),
        ninodes: u32::from_be_bytes([(*img)[bsize+15], (*img)[bsize+14], (*img)[bsize+13], (*img)[bsize+12]]),
        nlog: u32::from_be_bytes([(*img)[bsize+19], (*img)[bsize+18], (*img)[bsize+17], (*img)[bsize+16]]),
        logstart: u32::from_be_bytes([(*img)[bsize+23], (*img)[bsize+22], (*img)[bsize+21], (*img)[bsize+20]]),
        inodestart: u32::from_be_bytes([(*img)[bsize+27], (*img)[bsize+26], (*img)[bsize+25], (*img)[bsize+24]]),
        bmapstart: u32::from_be_bytes([(*img)[bsize+31], (*img)[bsize+30], (*img)[bsize+29], (*img)[bsize+28]]),
    };
    println!("{:?}", superblock);
}

#[derive(Debug)]
struct SuperBlock {
    magic: u32,        // Must be FSMAGIC
    size: u32,         // Size of file system image (blocks)
    nblocks: u32,      // Number of data blocks
    ninodes: u32,      // Number of inodes.
    nlog: u32,         // Number of log blocks
    logstart: u32,     // Block number of first log block
    inodestart: u32,   // Block number of first inode block
    bmapstart: u32,    // Block number of first free map block
  }