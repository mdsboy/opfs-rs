use libc;
use memmap::MmapOptions;
use std::env;
use std::ffi::CString;
use std::fs::File;
use std::mem;
use std::os::unix::io::{AsRawFd, FromRawFd};

use std::io::prelude::*;
use std::io::BufReader;

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
    let img = unsafe { MmapOptions::new().map(&file).unwrap() };
    println!("{:?}", img);
    /*
        let inner: Vec<u8> = img.to_vec();
        println!("{:?}", inner);
        println!("{}", inner.len());
    */
    //let ser = bincode::deserialize(&img).unwrap();
    /*let sblk = unsafe {
        std::mem::transmute::<std::ffi::c_void, superblock>(img.as_ptr())
    };*/
    println!("{}", img.first().unwrap());
    let sblk = SuperBlock {
        magic: u32::from_be_bytes([
            img[BSIZE + 3],
            img[BSIZE + 2],
            img[BSIZE + 1],
            img[BSIZE + 0],
        ]),
        size: u32::from_be_bytes([
            img[BSIZE + 7],
            img[BSIZE + 6],
            img[BSIZE + 5],
            img[BSIZE + 4],
        ]),
        nblocks: u32::from_be_bytes([
            img[BSIZE + 11],
            img[BSIZE + 10],
            img[BSIZE + 9],
            img[BSIZE + 8],
        ]),
        ninodes: u32::from_be_bytes([
            img[BSIZE + 15],
            img[BSIZE + 14],
            img[BSIZE + 13],
            img[BSIZE + 12],
        ]),
        nlog: u32::from_be_bytes([
            img[BSIZE + 19],
            img[BSIZE + 18],
            img[BSIZE + 17],
            img[BSIZE + 16],
        ]),
        logstart: u32::from_be_bytes([
            img[BSIZE + 23],
            img[BSIZE + 22],
            img[BSIZE + 21],
            img[BSIZE + 20],
        ]),
        inodestart: u32::from_be_bytes([
            img[BSIZE + 27],
            img[BSIZE + 26],
            img[BSIZE + 25],
            img[BSIZE + 24],
        ]),
        bmapstart: u32::from_be_bytes([
            img[BSIZE + 31],
            img[BSIZE + 30],
            img[BSIZE + 29],
            img[BSIZE + 28],
        ]),
    };
    println!("{:?}", sblk);

    let root_inode_number = 1;

    let ipb = BSIZE / mem::size_of::<Dinode>();
    println!("IPB:{}", ipb);
    let pos = root_inode_number / ipb + sblk.inodestart as usize;
    println!("{}", pos);
    /*let root_inode = img[pos];
    println!("{}", root_inode);*/
    let offset = (root_inode_number % ipb) as u8;
    println!("{}", offset);

    let inode_pos = BSIZE * pos + mem::size_of::<Dinode>();

    let mut root_inode = Dinode {
        file_type: i16::from_be_bytes([img[inode_pos + 1], img[inode_pos + 0]]),
        major: i16::from_be_bytes([img[inode_pos + 3], img[inode_pos + 2]]),
        minor: i16::from_be_bytes([img[inode_pos + 5], img[inode_pos + 4]]),
        nlink: i16::from_be_bytes([img[inode_pos + 7], img[inode_pos + 6]]),
        size: u32::from_be_bytes([
            img[inode_pos + 11],
            img[inode_pos + 10],
            img[inode_pos + 9],
            img[inode_pos + 8],
        ]),
        addrs: [0; NDIRECT + 1],
    };
    for i in 0..NDIRECT + 1 {
        root_inode.addrs[i] = u32::from_be_bytes([
            img[inode_pos + 12 + i * 4 + 3],
            img[inode_pos + 12 + i * 4 + 2],
            img[inode_pos + 12 + i * 4 + 1],
            img[inode_pos + 12 + i * 4 + 0],
        ]);
    }
    println!("{:?}", root_inode);
}

#[derive(Debug)]
struct SuperBlock {
    magic: u32,      // Must be FSMAGIC
    size: u32,       // Size of file system image (blocks)
    nblocks: u32,    // Number of data blocks
    ninodes: u32,    // Number of inodes.
    nlog: u32,       // Number of log blocks
    logstart: u32,   // Block number of first log block
    inodestart: u32, // Block number of first inode block
    bmapstart: u32,  // Block number of first free map block
}

const BSIZE: usize = 1024;
const NDIRECT: usize = 12;

#[derive(Debug)]
#[repr(C)]
struct Dinode {
    file_type: i16,            // File type
    major: i16,                // Major device number (T_DEVICE only)
    minor: i16,                // Minor device number (T_DEVICE only)
    nlink: i16,                // Number of links to inode in file system
    size: u32,                 // Size of file (bytes)
    addrs: [u32; NDIRECT + 1], // Data block addresses
}
