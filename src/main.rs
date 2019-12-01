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
    let img = unsafe { MmapOptions::new().map(&file).unwrap() };
    println!("{:?}", img);
    let sblk = get_superblock(&img);
    println!("{:?}", sblk);

    let root_inode_number = 1;
    let root_inode = iget(&img, &sblk, root_inode_number);

    match &**cmd {
        "ls" => {
            println!("ls");
            do_ls(&img, &root_inode, args.len() - 3, &args[3..]);
        }
        _ => {
            unimplemented!();
        }
    }
}

fn get_superblock(img: &memmap::Mmap) -> SuperBlock {
    SuperBlock {
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
    }
}

const IPB: usize = BSIZE / mem::size_of::<Dinode>();

fn iget(img: &memmap::Mmap, sblk: &SuperBlock, inum: usize) -> Dinode {
    //println!("IPB:{}", IPB);
    let pos = inum / IPB + sblk.inodestart as usize;
    //println!("{}", pos);
    let offset = inum % IPB;
    //println!("{}", offset);

    let inode_pos = BSIZE * pos + mem::size_of::<Dinode>() * offset;

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
    //println!("{:?}", root_inode);
    return root_inode;
}

const DIRSIZ: usize = 14;
/*
fn skiplem(path: &String, name: &String) -> String {
    let mut i = 0;
    let path: Vec<char> = path.chars().collect();
    while path[i] == '/' {
        i += 1;
    }
    path
}
*/

/*
fn bmap(img: &memmap::Mmap, ip: &Dinode, n: usize) {
    if n < NDIRECT {
        let addr = ip.addrs[n];
        if addr == 0 {
            addr =
        }
    }
}*/

fn iread(img: &memmap::Mmap, ip: &Dinode, n: usize, off: usize) -> Vec<u8> {
    /*let mut t = 0;
    let mut m = 0;
    let mut off = off;*/
    //while t < n {
    let mut buf = vec![0; n];
    (&img[BSIZE * ip.addrs[(off / BSIZE) as usize] as usize + off % BSIZE..])
        .read_exact(buf.as_mut_slice())
        .unwrap();
    /*
    let po = ip.addrs[(off / BSIZE) as usize] as usize;
    println!("addr:{}", po);
    println!("off:{}", off);
    println!("{:?}", img[po..po+10].to_vec());*/
    /*m = std::cmp::min(n - t, BSIZE - off % BSIZE);
        t += m;
        off += m;
    }
    vec![]*/
    buf
}
fn dlookup(img: &memmap::Mmap, dp: &Dinode, name: &String) -> Option<Dinode> {
    let mut off = 0;
    while off < dp.size {
        let buf = iread(&img, dp, std::mem::size_of::<Dirent>(), off as usize);
        //println!("{:?}", buf);

        let mut de = Dirent {
            inum: u16::from_be_bytes([buf[1], buf[0]]),
            name: [0; DIRSIZ],
        };

        for i in 0..DIRSIZ {
            de.name[i] = buf[i + 2];
        }
        let search_name = &de
            .name
            .iter()
            .filter(|&c| *c != 0)
            .map(|&c| c as char)
            .collect::<String>();
        //println!("{},{}", name, search_name);
        if name == search_name {
            //println!("po");
            return Some(iget(img, &get_superblock(img), de.inum as usize));
        }

        off += mem::size_of::<Dinode>() as u32;
    }
    None
}

fn ilookup(img: &memmap::Mmap, rp: &Dinode, path: &String) -> Option<Dinode> {
    let names: Vec<&str> = path.split('/').filter(|s| *s != "").collect();
    //println!("{:?}", names);
    let mut rp = (*rp).clone();
    if names.is_empty() {
        return Some(rp);
    }

    for i in 0..names.len() {
        //println!("{}", names[i]);
        rp = match dlookup(img, &rp, &names[i].to_string()) {
            Some(dp) => dp,
            None => return None,
        };
    }
    Some(rp)
}

const T_DIR: i16 = 1;

fn do_ls(img: &memmap::Mmap, root_inode: &Dinode, argc: usize, argv: &[String]) {
    if argc != 1 {
        println!("error");
        return;
    }
    let path = &argv[0];
    let ip = match ilookup(img, root_inode, &path) {
        Some(ip) => ip,
        None => {
            println!("error");
            return;
        }
    };
    println!("{}", path);
    println!("{:?}", ip);
    if ip.file_type == T_DIR {
        let mut off = 0;
        while off < ip.size {
            //let mut buf = vec![0; std::mem::size_of::<Dinode>()];
            let buf = iread(&img, &ip, std::mem::size_of::<Dirent>(), off as usize);
            //println!("{:?}", buf);

            let mut de = Dirent {
                inum: u16::from_be_bytes([buf[1], buf[0]]),
                name: [0; DIRSIZ],
            };

            for i in 0..DIRSIZ {
                de.name[i] = buf[i + 2];
            }

            off += std::mem::size_of::<Dirent>() as u32;
            if de.inum == 0 {
                continue;
            }

            let p = iget(img, &get_superblock(img), de.inum as usize);
            println!(
                "{}, {}, {}, {}",
                de.name.iter().map(|&c| c as char).collect::<String>(),
                p.file_type,
                de.inum,
                p.size
            );
            //println!("{:?}", p);

            //println!("{}", off);
        }
    } else {
    }
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

#[derive(Debug, Clone)]
#[repr(C)]
struct Dinode {
    file_type: i16,            // File type
    major: i16,                // Major device number (T_DEVICE only)
    minor: i16,                // Minor device number (T_DEVICE only)
    nlink: i16,                // Number of links to inode in file system
    size: u32,                 // Size of file (bytes)
    addrs: [u32; NDIRECT + 1], // Data block addresses
}

#[derive(Debug)]
struct Dirent {
    inum: u16,
    name: [u8; DIRSIZ],
}
