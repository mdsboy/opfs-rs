use std::mem;

pub const IPB: usize = BSIZE / mem::size_of::<Dinode>();

pub const DIRSIZ: usize = 14;
pub const T_DIR: i16 = 1;

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