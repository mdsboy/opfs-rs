use std::mem;

pub const INODE_SIZE: usize = 64;
pub const DIRENT_SIZE: usize = mem::size_of::<Dirent>();

pub const IPB: usize = BSIZE / INODE_SIZE;

pub const DIRSIZ: usize = 14;

pub const T_DIR: i16 = 1; // Directory
pub const T_FILE: i16 = 2; // File
pub const T_DEV: i16 = 3; // Device


pub const MAXFILE: usize = NDIRECT + NINDIRECT;
pub const MAXFILESIZE: usize = MAXFILE * BSIZE;

pub const BSIZE: usize = 1024;
pub const NDIRECT: usize = 12;
pub const NINDIRECT: usize = BSIZE / mem::size_of::<u32>();
pub const BUFSIZE: usize = 1024;

#[derive(Debug)]
pub struct SuperBlock {
    pub magic: u32,      // Must be FSMAGIC
    pub size: u32,       // Size of file system image (blocks)
    pub nblocks: u32,    // Number of data blocks
    pub ninodes: u32,    // Number of inodes.
    pub nlog: u32,       // Number of log blocks
    pub logstart: u32,   // Block number of first log block
    pub inodestart: u32, // Block number of first inode block
    pub bmapstart: u32,  // Block number of first free map block
}

#[derive(Debug, Clone)]
pub struct Dinode {
    pub pos: usize,
    pub inum: u16,
    pub file_type: i16,            // File type
    pub major: i16,                // Major device number (T_DEVICE only)
    pub minor: i16,                // Minor device number (T_DEVICE only)
    pub nlink: i16,                // Number of links to inode in file system
    pub size: u32,                 // Size of file (bytes)
    pub addrs: [u32; NDIRECT + 1], // Data block addresses
}

#[derive(Debug)]
pub struct Dirent {
    pub inum: u16,
    pub name: [u8; DIRSIZ],
}
