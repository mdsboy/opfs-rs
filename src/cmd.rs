use crate::fs::*;
use crate::libfs::*;
use std::fs::File;
use std::io::BufWriter;
use std::io::{Read, Write};

pub fn do_ls(img: &Vec<u8>, root_inode: &Dinode, argc: usize, argv: &[String]) {
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
    }; /*
       println!("{}", path);
       println!("{:?}", ip);*/
    if ip.file_type == T_DIR {
        let mut off = 0;
        while off < ip.size {
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
                "{} {} {} {}",
                de.name.iter().map(|&c| c as char).collect::<String>(),
                p.file_type,
                de.inum,
                p.size
            );
            //println!("{:?}", p);

            //println!("{}", off);
        }
    } else {
        println!("{} {} {} {}", path, ip.file_type, ip.inum, ip.size);
    }
}

pub fn do_get(img: &Vec<u8>, root_inode: &Dinode, argc: usize, argv: &[String]) {
    if argc != 2 {
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

    println!("{}", ip.size);
    //let buf = iread(&img, &ip, ip.size as usize, 0);
    /*
    println!(
        "{}",
        buf.iter()
            .map(|&c| c as char)
            .collect::<String>()
    );*/

    let path2 = &argv[1];
    let mut writer = BufWriter::new(File::create(path2).unwrap());
    //println!("{}", std::str::from_utf8(&buf).unwrap());
    
    let mut off = 0;
    while off < ip.size {
        let buf = iread(&img, &ip, BUFSIZE, off as usize);
        //println!("{}", buf.iter().map(|&c| c as char).collect::<String>());
        //println!("{:?}", buf);
        println!("{}", std::str::from_utf8(&buf).unwrap());
        writer.write_all(&buf).unwrap();

        off += BUFSIZE as u32;
    }
}

pub fn do_put(img: &mut Vec<u8>, root_inode: &Dinode, argc: usize, argv: &[String], file_name: &String) {
    if argc != 2 {
        println!("error");
        return;
    }

    let path = &argv[0];
    let path2 = &argv[1];
    let mut file = File::open(path).unwrap();
    let mut buf: Vec<u8> = vec![];
    file.read_to_end(&mut buf).unwrap();
    println!("{}", std::str::from_utf8(&buf).unwrap());

    let mut ip = match ilookup(img, root_inode, &path2) {
        Some(ip) => {
            println!("found!");
            if ip.file_type != T_FILE {
                println!("error");
                return;
            } else {
                ip
            }
        }
        None => match icreate(img, root_inode, path2) {
            Some(ip) => ip,
            None => {
                println!("error");
                return;
            }
        },
    };
    println!("{}", ip.size);

    // let mut off = 0;
    // while off < MAXFILESIZE {
    //     iwrite(img, &ip, BUFSIZE, off as usize, &buf);
    //     //println!("{}", buf.iter().map(|&c| c as char).collect::<String>());
    //     //println!("{:?}", buf);
    //     println!("{}", std::str::from_utf8(&buf).unwrap());

    //     off += BUFSIZE;
    // }

    iwrite(img, &mut ip, 0, &buf);
    println!("{}", std::str::from_utf8(&buf).unwrap());

    let mut writer = File::create(file_name).unwrap();
    writer.write_all(&img).unwrap();

    //let buf = iread(&img, &ip, ip.size as usize, 0);
    /*
    println!(
        "{}",
        buf.iter()
            .map(|&c| c as char)
            .collect::<String>()
    );*/

    //println!("{}", std::str::from_utf8(&buf).unwrap());
}
