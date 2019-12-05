use crate::fs::*;

pub fn get_superblock(img: &Vec<u8>) -> SuperBlock {
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

pub fn iget(img: &Vec<u8>, sblk: &SuperBlock, inum: usize) -> Dinode {
    /*println!("IPB:{}", IPB);
    println!("inum:{}", inum);*/
    let pos = inum / IPB + sblk.inodestart as usize;
    //println!("{}", pos);
    let offset = inum % IPB;
    //println!("{}", offset);

    let inode_pos = BSIZE * pos + INODE_SIZE * offset;
    //println!("ipos:{}", inode_pos);

    let mut root_inode = Dinode {
        pos: inode_pos,
        inum: inum as u16,
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

pub fn bmap(img: &Vec<u8>, ip: &Dinode, n: usize) -> usize {
    //println!("n:{}", n);
    if n < NDIRECT {
        //println!("addr:{}", ip.addrs[n]);
        return BSIZE * ip.addrs[n] as usize;
    } else {
        let k = n - NDIRECT;
        if k >= NINDIRECT {
            println!("error");
            return 0;
        }
        let iaddr = ip.addrs[NDIRECT];
        let pos = (iaddr as usize) * BSIZE + k * 4;
        return BSIZE
            * u32::from_be_bytes([img[pos + 3], img[pos + 2], img[pos + 1], img[pos + 0]])
                as usize;
    }
}

pub fn iwrite(img: &mut Vec<u8>, ip: &Dinode, off: usize, buf: &Vec<u8>) {
    //let n = if off + n > ip.size as usize { ip.size as usize - off } else { n };
    let mut off = off;
    let mut t = 0;
    let n = buf.len();
    println!("n:{}", n);
    println!("off:{}", off);
    while t < n {
        let pos = bmap(img, ip, off / BSIZE) + off % BSIZE;
        let m = std::cmp::min(n - t, BSIZE - off % BSIZE);
        //println!("{}, {}", pos, m);
        //println!("{}", off);
        //buf.extend_from_slice(&img[pos..pos+m]);
        for i in 0..m {
            img[pos + i] = buf[t + i];
            println!("{}:{}", i, buf[t + i] as char);
        }

        t += m;
        off += m;
    }
    if t > 0 && off as u32 > ip.size {
        let bytes = (off as u32).to_le_bytes();
        println!("{:?}", bytes);
        img[ip.pos + 11] = bytes[3];
        img[ip.pos + 10] = bytes[2];
        img[ip.pos + 9] = bytes[1];
        img[ip.pos + 8] = bytes[0];
    }
}

pub fn iread(img: &Vec<u8>, ip: &Dinode, n: usize, off: usize) -> Vec<u8> {
    let n = if off + n > ip.size as usize {
        ip.size as usize - off
    } else {
        n
    };
    let mut buf = vec![];
    let mut off = off;
    let mut t = 0;
    while t < n {
        let pos = bmap(img, ip, off / BSIZE) + off % BSIZE;
        let m = std::cmp::min(n - t, BSIZE - off % BSIZE);
        //println!("{}, {}", pos, m);
        //println!("{}", off);
        buf.extend_from_slice(&img[pos..pos + m]);

        t += m;
        off += m;
    }
    buf
}

pub fn dlookup(img: &Vec<u8>, dp: &Dinode, name: &String) -> Option<(Dinode, usize)> {
    let mut off = 0;
    while off < dp.size {
        let buf = iread(&img, dp, DIRENT_SIZE, off as usize);
        //println!("{:?}", buf);

        let mut de = Dirent {
            inum: u16::from_be_bytes([buf[1], buf[0]]),
            name: [0; DIRSIZ],
        };

        for i in 0..DIRSIZ {
            if i + 2 < buf.len() {
                de.name[i] = buf[i + 2];
            }
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
            return Some((
                iget(img, &get_superblock(img), de.inum as usize),
                off as usize,
            ));
        }

        off += DIRENT_SIZE as u32;
    }
    None
}

pub fn ilookup(img: &Vec<u8>, rp: &Dinode, path: &String) -> Option<Dinode> {
    let names: Vec<&str> = path.split('/').filter(|s| *s != "").collect();
    //println!("{:?}", names);
    let mut rp = (*rp).clone();
    if names.is_empty() {
        return Some(rp);
    }

    for i in 0..names.len() {
        //println!("{}", names[i]);
        rp = match dlookup(img, &rp, &names[i].to_string()) {
            Some((dp, _)) => dp,
            None => return None,
        };
    }
    Some(rp)
}

pub fn icreate(img: &mut Vec<u8>, rp: &Dinode, path: &String) -> Option<Dinode> {
    println!("not found!");
    let names: Vec<&str> = path.split('/').filter(|s| *s != "").collect();
    //println!("{:?}", names);
    let mut rp = (*rp).clone();
    if names.is_empty() {
        return Some(rp);
    }

    for i in 0..names.len() {
        //println!("{}", names[i]);
        rp = match dlookup(img, &rp, &names[i].to_string()) {
            Some(_) => return None,
            None => {
                let mut ip = match ialloc(img, T_FILE) {
                    Some(ip) => ip,
                    None => return None,
                };
                daddent(img, &rp, &names[i].to_string(), &mut ip);
                if ip.file_type == T_DIR {
                    daddent(img, &rp, &String::from("."), &mut ip);
                    daddent(img, &rp, &String::from(".."), &mut ip);
                }
                ip
            }
        };
    }
    Some(rp)
}

pub fn ialloc(img: &mut Vec<u8>, file_type: i16) -> Option<Dinode> {
    let sblk = get_superblock(img);
    for inum in 1..sblk.ninodes {
        let mut inode = iget(img, &get_superblock(img), inum as usize);
        if inode.file_type == 0 {
            inode.file_type = file_type;
            let inum = inum as usize;
            let pos = inum / IPB + sblk.inodestart as usize;
            let offset = inum % IPB;
            let inode_pos = BSIZE * pos + INODE_SIZE * offset;
            let bytes = (file_type as u32).to_le_bytes();
            img[inode_pos + 1] = bytes[1];
            img[inode_pos + 0] = bytes[0];
            return Some(inode);
        }
    }
    None
}

pub fn daddent(img: &mut Vec<u8>, dp: &Dinode, name: &String, ip: &mut Dinode) {
    let mut de = Dirent {
        inum: 0,
        name: [0; DIRSIZ],
    };
    let mut off = 0;
    //let mut name: Vec<char> = vec![];

    while off < dp.size {
        let buf = iread(img, dp, DIRENT_SIZE, off as usize);

        de = Dirent {
            inum: u16::from_be_bytes([buf[1], buf[0]]),
            name: [0; DIRSIZ],
        };

        let name_chars: Vec<char> = name.chars().collect();
        for i in 0..std::cmp::min(DIRSIZ, name_chars.len()) {
            de.name[i] = name_chars[i] as u8;
        }

        if de.inum == 0 {
            break;
        }

        off += DIRENT_SIZE as u32;
    }
    de.inum = ip.inum;

    let mut buf: Vec<u8> = de.inum.to_le_bytes().to_vec();
    buf.append(&mut de.name.to_vec());
    iwrite(img, dp, off as usize, &buf);

    if de.name.len() != 1 || de.name[0] != ('.' as u8) {
        ip.nlink += 1;
    }
}

pub fn iunlink(img: &mut Vec<u8>, rp: &Dinode, path: &String) {
    let names: Vec<&str> = path.split('/').filter(|s| *s != "").collect();
    //println!("{:?}", names);
    let mut rp = (*rp).clone();

    for i in 0..names.len() {
        //println!("{}", names[i]);
        let name = &names[i].to_string();
        if name == "." || name == ".." {
            return;
        }
        let (mut ip, off) = match dlookup(img, &rp, name) {
            Some((ip, off)) => (ip, off),
            None => return,
        };
        if i < names.len() - 1 {
            rp = ip;
            continue;
        }
        println!("{}", names[i]);
        println!("{:?}", ip);

        let zero: Vec<u8> = vec![0; DIRENT_SIZE];
        println!("off:{}", off);
        println!("size:{}", DIRENT_SIZE);
        iwrite(img, &rp, off, &zero);
        /*
        if let Some((rpp, _)) = dlookup(img, &ip, &"..".to_string()) {
            if ip.file_type == T_DIR && rpp.pos == rp.pos {
                let bytes = (rp.nlink as i16).to_le_bytes();
                img[rp.pos + 7] = bytes[1];
                img[rp.pos + 6] = bytes[0];
                rp.nlink -= 1;
            }
        }
        let bytes = (ip.nlink as i16).to_le_bytes();
        img[ip.pos + 7] = bytes[1];
        img[ip.pos + 6] = bytes[0];
        ip.nlink -= 1;
        if ip.nlink == 0 {
            if ip.file_type != T_DEV {
                itruncate(img, &ip);
            }
            for i in 0..ip.size {
                img[ip.pos + i as usize] = 0;
            }
        }*/
    }
}

pub fn itruncate(img: &mut Vec<u8>, ip: &Dinode) {
    let n = (ip.size as usize + BSIZE - 1) / BSIZE;
    println!("{}", n);
    for i in 0..std::cmp::min(n, NDIRECT) {
        for j in 0..BSIZE {
            img[ip.addrs[i] as usize * BSIZE + j] = 0;
        }
    }
    if n > NDIRECT {
        let iaddr = ip.addrs[NDIRECT];
        let ni = n - NDIRECT;
        for i in 0..ni {
            let pos = bmap(img, ip, i);
            for j in 0..BSIZE {
                img[pos as usize + j] = 0;
            }
        }
        for j in 0..BSIZE {
            img[iaddr as usize * BSIZE + j] = 0;
        }
    }
}
