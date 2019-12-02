use std::env;
use std::io::prelude::*;

mod libfs;
mod cmd;
mod fs;
use libfs::*;

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
    let mut img = Vec::new();
    (&file).read_to_end(&mut img).unwrap();
    //println!("{:?}", img);

    let sblk = get_superblock(&img);
    let root_inode_number = 1;
    let root_inode = iget(&img, &sblk, root_inode_number);

    match &**cmd {
        "ls" => {
            println!("ls");
            cmd::do_ls(&img, &root_inode, args.len() - 3, &args[3..]);
        }
        "get" => {
            println!("get");
            cmd::do_get(&img, &root_inode, args.len() - 3, &args[3..]);
        }
        "put" => {
            println!("put");
            cmd::do_put(&mut img, &root_inode, args.len() - 3, &args[3..], &img_file);
        }
        "rn" => {
            println!("rm");
            unimplemented!();
        }
        _ => {
            println!("error");
        }
    }
}
