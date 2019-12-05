use crate::fs::*;

pub fn get_superblock(img: &Vec<u8>) -> SuperBlock {
    SuperBlock {
        magic: get_u32(img, BSIZE),
        size: get_u32(img, BSIZE + 4),
        nblocks: get_u32(img, BSIZE + 8),
        ninodes: get_u32(img, BSIZE + 12),
        nlog: get_u32(img, BSIZE + 16),
        logstart: get_u32(img, BSIZE + 20),
        inodestart: get_u32(img, BSIZE + 24),
        bmapstart: get_u32(img, BSIZE + 28),
    }
}

pub fn get_u32(img: &Vec<u8>, pos: usize) -> u32 {
    u32::from_le_bytes([img[pos], img[pos + 1], img[pos + 2], img[pos + 3]])
}

pub fn get_i16(img: &Vec<u8>, pos: usize) -> i16 {
    i16::from_le_bytes([img[pos], img[pos + 1]])
}

pub fn get_inode_pos(img: &Vec<u8>, inum: usize) -> usize {
    let sblk = get_superblock(img);
    let pos = inum / IPB + sblk.inodestart as usize;
    let offset = inum % IPB;

    BSIZE * pos + INODE_SIZE * offset
}

// returns the inum-th Dinode structure
pub fn iget(img: &Vec<u8>, inum: usize) -> Dinode {
    let inode_pos = get_inode_pos(img, inum);

    let mut root_inode = Dinode {
        pos: inode_pos,
        inum: inum as u16,
        file_type: get_i16(img, inode_pos),
        major: get_i16(img, inode_pos + 2),
        minor: get_i16(img, inode_pos + 4),
        nlink: get_i16(img, inode_pos + 6),
        size: get_u32(img, inode_pos + 8),
        addrs: [0; NDIRECT + 1],
    };
    for i in 0..NDIRECT + 1 {
        root_inode.addrs[i] = get_u32(img, inode_pos + 12 + i * 4);
    }

    return root_inode;
}

// returns n-th data block number of the file specified by ip
pub fn bmap(img: &Vec<u8>, ip: &Dinode, n: usize) -> usize {
    if n < NDIRECT {
        return BSIZE * ip.addrs[n] as usize;
    } else {
        let k = n - NDIRECT;
        if k >= NINDIRECT {
            println!("error");
            return 0;
        }
        let iaddr = ip.addrs[NDIRECT];
        let pos = (iaddr as usize) * BSIZE + k * 4;
        return BSIZE * get_u32(img, pos) as usize;
    }
}

// writes buf to the file specified by ip
pub fn iwrite(img: &mut Vec<u8>, ip: &Dinode, off: usize, buf: &Vec<u8>) -> usize {
    let mut off = off;
    let mut t = 0;
    let n = buf.len();
    while t < n {
        let pos = bmap(img, ip, off / BSIZE) + off % BSIZE;
        let m = std::cmp::min(n - t, BSIZE - off % BSIZE);
        for i in 0..m {
            img[pos + i] = buf[t + i];
            println!("{}:{}", i, buf[t + i] as char);
        }

        t += m;
        off += m;
    }
    if t > 0 && off > ip.size as usize {
        write_u32(img, ip.pos + 8, off as u32);
    }
    off
}

pub fn write_u32(img: &mut Vec<u8>, pos: usize, data: u32) {
    let bytes = data.to_le_bytes();
    img[pos] = bytes[0];
    img[pos + 1] = bytes[1];
    img[pos + 2] = bytes[2];
    img[pos + 3] = bytes[3];
}

// reads n byte of data from the file specified by ip
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
        buf.extend_from_slice(&img[pos..pos + m]);

        t += m;
        off += m;
    }
    buf
}

pub fn get_dirent(buf: &Vec<u8>) -> Dirent {
    let mut de = Dirent {
        inum: u16::from_be_bytes([buf[1], buf[0]]),
        name: [0; DIRSIZ],
    };

    for i in 0..DIRSIZ {
        if i + 2 < buf.len() {
            de.name[i] = buf[i + 2];
        }
    }
    de
}

// search a file (name) in a directory (dp)
pub fn dlookup(img: &Vec<u8>, dp: &Dinode, name: &String) -> Option<(Dinode, usize)> {
    let mut off = 0;
    while off < dp.size {
        let mut buf = iread(&img, dp, DIRENT_SIZE, off as usize);
        buf.append(&mut vec![0_u8; DIRENT_SIZE]);
        let de = get_dirent(&buf);

        let search_name = &de
            .name
            .iter()
            .filter(|&c| *c != 0)
            .map(|&c| c as char)
            .collect::<String>();

        if name == search_name {
            return Some((iget(img, de.inum as usize), off as usize));
        }

        off += DIRENT_SIZE as u32;
    }
    None
}

// returns the inode number of a file (rp/path)
pub fn ilookup(img: &Vec<u8>, rp: &Dinode, path: &String) -> Option<Dinode> {
    let names: Vec<&str> = path.split('/').filter(|s| *s != "").collect();
    let mut rp = (*rp).clone();
    if names.is_empty() {
        return Some(rp);
    }

    for i in 0..names.len() {
        rp = match dlookup(img, &rp, &names[i].to_string()) {
            Some((dp, _)) => dp,
            None => return None,
        };
    }
    Some(rp)
}

// create a file
pub fn icreate(img: &mut Vec<u8>, rp: &Dinode, path: &String) -> Option<Dinode> {
    let names: Vec<&str> = path.split('/').filter(|s| *s != "").collect();
    let mut rp = (*rp).clone();
    if names.is_empty() {
        return Some(rp);
    }

    for i in 0..names.len() {
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

// allocate a new inode structure
pub fn ialloc(img: &mut Vec<u8>, file_type: i16) -> Option<Dinode> {
    let sblk = get_superblock(img);
    for inum in 1..sblk.ninodes {
        let mut inode = iget(img, inum as usize);
        if inode.file_type == 0 {
            inode.file_type = file_type;
            let inode_pos = get_inode_pos(img, inum as usize);
            let bytes = (file_type as u32).to_le_bytes();
            img[inode_pos + 1] = bytes[1];
            img[inode_pos + 0] = bytes[0];
            return Some(inode);
        }
    }
    None
}

// add a new directory entry in dp
pub fn daddent(img: &mut Vec<u8>, dp: &Dinode, name: &String, ip: &mut Dinode) {
    let mut de = Dirent {
        inum: 0,
        name: [0; DIRSIZ],
    };
    let mut off = 0;

    while off < dp.size {
        let mut buf = iread(img, dp, DIRENT_SIZE, off as usize);
        buf.append(&mut vec![0_u8; DIRENT_SIZE]);

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
    let mut rp = (*rp).clone();

    for i in 0..names.len() {
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

        let zero: Vec<u8> = vec![0; DIRENT_SIZE];
        iwrite(img, &rp, off, &zero);
        println!("PO1");

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
                itruncate(img, &mut ip);
            }
            for i in 0..ip.size {
                img[ip.pos + i as usize] = 0;
            }
        }
    }
}

// truncate the file specified by ip to size
pub fn itruncate(img: &mut Vec<u8>, ip: &mut Dinode) {
    let n = (ip.size as usize + BSIZE - 1) / BSIZE;
    println!("{}", n);
    for i in 0..std::cmp::min(n, NDIRECT) {
        for j in 0..BSIZE {
            img[ip.addrs[i] as usize * BSIZE + j] = 0;
        }
        ip.addrs[i] = 0;
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
        ip.addrs[NDIRECT] = 0;
    }
    write_u32(img, ip.pos + 8, 0);
    ip.size = 0;
}
