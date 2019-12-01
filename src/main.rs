use std::env;
use std::io::prelude::*;

mod libfs;
mod cmd;
use libfs::*;
use cmd::*;

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
    let _ = (&file).read_to_end(&mut img).unwrap();
    //println!("{:?}", img);

    let sblk = get_superblock(&img);
    let root_inode_number = 1;
    let root_inode = iget(&img, &sblk, root_inode_number);

    match &**cmd {
        "ls" => {
            println!("ls");
            do_ls(&img, &root_inode, args.len() - 3, &args[3..]);
        }
        "get" => {
            println!("get");
            unimplemented!();
        }
        "put" => {
            println!("put");
            unimplemented!();
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
