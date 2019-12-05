use std::env;
use std::fs::File;
use std::io::prelude::*;

mod cmd;
mod fs;
mod libfs;
use libfs::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("error");
        return;
    }

    let progname = &args[0];
    let img_file = &args[1];
    let cmd = &args[2];

    let file = File::open(img_file).unwrap();
    let mut img = Vec::new();
    (&file).read_to_end(&mut img).unwrap(); 

    let root_inode = iget(&img, fs::ROOT_INODE_NUMBER);

    match &**cmd {
        "ls" => {
            println!("ls");
            cmd::do_ls(&img, &root_inode, &args[3..]);
        }
        "get" => {
            println!("get");
            cmd::do_get(&img, &root_inode, &args[3..]);
        }
        "put" => {
            println!("put");
            cmd::do_put(&mut img, &root_inode, &args[3..]);
        }
        "rm" => {
            println!("rm");
            cmd::do_rm(&mut img, &root_inode, &args[3..]);
        }
        _ => {
            println!("error");
        }
    }

    let mut writer = std::fs::File::create(&img_file).unwrap();
    writer.write_all(&img).unwrap();
}
