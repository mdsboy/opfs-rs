use crate::fs::*;
use crate::libfs::*;
use std::fs::File;
use std::io::BufWriter;
use std::io::{Read, Write};

// ls path
pub fn do_ls(img: &Vec<u8>, root_inode: &Dinode, argv: &[String], progname: &String) {
    if argv.len() != 1 {
        eprintln!("usage: {} img_file ls path", progname);
        return;
    }

    let path = &argv[0];
    let ip = match ilookup(img, root_inode, &path) {
        Some(ip) => ip,
        None => {
            eprintln!("ls: {}: no such file or directory", path);
            return;
        }
    };

    if ip.file_type == T_DIR {
        let mut off = 0;
        while off < ip.size {
            let buf = iread(&img, &ip, std::mem::size_of::<Dirent>(), off as usize);

            let de = get_dirent(&buf);

            off += std::mem::size_of::<Dirent>() as u32;
            if de.inum == 0 {
                continue;
            }

            let p = iget(img, de.inum as usize);
            println!(
                "{} {} {} {}",
                de.name.iter().map(|&c| c as char).collect::<String>(),
                p.file_type,
                de.inum,
                p.size
            );
        }
    } else {
        println!("{} {} {} {}", path, ip.file_type, ip.inum, ip.size);
    }
}

// get path
pub fn do_get(img: &Vec<u8>, root_inode: &Dinode, argv: &[String], progname: &String) {
    if argv.len() != 2 {
        eprintln!("usage: {} img_file get path1 path2", progname);
        return;
    }
    let path = &argv[0];
    let ip = match ilookup(img, root_inode, &path) {
        Some(ip) => ip,
        None => {
            eprintln!("get: no such file or directory: {}", path);
            return;
        }
    };

    let path2 = &argv[1];
    let mut writer = BufWriter::new(File::create(path2).unwrap());

    let mut off = 0;
    while off < ip.size {
        let buf = iread(&img, &ip, BUFSIZE, off as usize);
        writer.write_all(&buf).unwrap();

        off += BUFSIZE as u32;
    }
}

// put path
pub fn do_put(img: &mut Vec<u8>, root_inode: &Dinode, argv: &[String], progname: &String) {
    if argv.len() != 2 {
        eprintln!("usage: {} img_file put path1 path2", progname);
        return;
    }

    let path = &argv[0];
    let path2 = &argv[1];
    let mut file = File::open(path).unwrap();
    let mut buf: Vec<u8> = vec![];
    file.read_to_end(&mut buf).unwrap();

    let mut ip = match ilookup(img, root_inode, &path2) {
        Some(mut ip) => {
            if ip.file_type != T_FILE {
                eprintln!("put: {}: directory or device", path2);
                return;
            } else {
                ip.size = 0;
                ip
            }
        }
        None => match icreate(img, root_inode, path2) {
            Some(ip) => ip,
            None => {
                eprintln!("put: {}: cannot create", path2);
                return;
            }
        },
    };

    iwrite(img, &mut ip, 0, &buf);
}

pub fn do_rm(img: &mut Vec<u8>, root_inode: &Dinode, argv: &[String], progname: &String) {
    if argv.len() != 1 {
        eprintln!("usage: {} img_file rm path", progname);
        return;
    }

    let path = &argv[0];
    let ip = match ilookup(img, root_inode, path) {
        Some(ip) => ip,
        None => {
            eprintln!("rm: {}: no such file or directory", path);
            return;
        }
    };
    if ip.file_type == T_DIR {
        eprintln!("rm: {}: a directory", path);
        return;
    }

    iunlink(img, root_inode, path);
}
