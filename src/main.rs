use std::{fs};
use std::path::{PathBuf};
use pad::{PadStr, Alignment};
use clap::{Arg, App};
use humansize::{FileSize, file_size_opts as options};
use std::str::FromStr;

struct DirStat {
    size: u64,
    typ: char,
    depth:u32,
    path: PathBuf,
    children: Vec<DirStat>
}

fn display_stat(ds: &DirStat, showdepth: u32, threshold:u32, parentsize:u64, ) {
    if ds.depth > showdepth {
        return;
    }

    if ds.depth == 0 {
        println!("{}:{}:{:3}:{} {}",
                 ds.depth,
                 ds.typ,
                 "100%",
                 ds.size.file_size(options::DECIMAL).unwrap().pad_to_width_with_alignment(10, Alignment::Right),
                 ds.path.display());
    } else {
        let pecent =  (((ds.size as f32) / (parentsize as f32)) * 100f32) as u32;

        print!("{}", "  ".repeat(ds.depth as usize));
        println!("{:3}:{}:{:3}%:{:10} {}",
                 ds.depth,
                 ds.typ,
                 pecent,
                 ds.size.file_size(options::DECIMAL).unwrap().pad_to_width_with_alignment(10, Alignment::Right),
                 ds.path.display());

    }

    for child in ds.children.iter() {
        let pecent =  (((child.size as f32) / (ds.size as f32)) * 100f32) as u32;
        if pecent >= threshold {
            display_stat(child, showdepth, threshold, ds.size);
        }
    }
}

fn calculate_dir_stat(depth:u32, root_dir:PathBuf) -> Option<DirStat> {
    let readdir = fs::read_dir(root_dir.as_path());
    let rdir = match readdir {
        Ok(o) => {o},
        Err(e) => { eprintln!("Error access ReadDir {}. {:?}", root_dir.as_path().display(), e);
            return None; },
    };

    let mut children:Vec::<DirStat> = Vec::new();
    let mut sum = 0u64;
    for item in rdir {
        // error handling
        let entry = match item {
            Ok(o) => {o},
            Err(e) => { eprintln!("Error access DirEntry {:?}", e); continue;},
        };

        // error handling
        let typ = match entry.file_type() {
            Ok(o) => {o},
            Err(e) => { eprintln!("Error access FileType {}. \n{:?}", entry.path().display(), e); continue; },
        };

        if typ.is_dir() {
            let option = calculate_dir_stat(depth+1, entry.path());
            match option {
                Some(ds) => {
                    sum += ds.size;
                    children.push(ds);
                },
                None => {/* no ds for this dir, do nothing */}
            }
        } else if typ.is_file(){
            let meta = match entry.metadata() {
                Ok(o) => o,
                Err(e) => {eprintln!("Error access Metadata {}. \n{:?}", entry.path().display(), e); continue;}
            };
            sum += meta.len();
            let ds = DirStat{
                size:meta.len(),
                typ:'F',
                depth:depth+1,
                path: entry.path(),
                children: Vec::new()
            };
            children.push(ds);
        } else if typ.is_symlink(){
            eprintln!("Ignore symlink {}", entry.path().display());
        }
    }

    // large size first
    children.sort_by(|x, y| x.size.partial_cmp(&y.size).unwrap().reverse());
    Some(DirStat {
        size: sum,
        typ: 'D',
        depth:depth,
        path: root_dir,
        children: children
    })
}

fn parse_args() -> (PathBuf, u32, u32) {
    let mut root = PathBuf::from_str(".").unwrap();
    let mut depth = 1u32;
    let mut threshold: u32 = 0;
    let matches = App::new("Show Item Percentage of Directory")
        .version("0.1.0")
        .author("Forrest Feng <changzheng.feng@carestream.com>")
        .about("Calcualte the item size % of your Directory")
        .arg(Arg::with_name("directory")
            .short("d")
            .long("dir")
            .value_name("ROOT_DIR")
            .help("Set the directory to calculate")
            .takes_value(true)
            .index(1)
            .multiple(false)
        )
        .arg(Arg::with_name("nest")
            .short("n")
            .long("nest")
            .value_name("DEPTH_LEVEL")
            .help("Set the nest level to display, eg. 3")
            .takes_value(true)
            .multiple(false))
        .arg(Arg::with_name("percent")
            .short("p")
            .long("percent")
            .value_name("PERCENT_THRESHOLD")
            .help("Set the percent threshold to display, eg. 10")
            .multiple(false))
        .get_matches();
// extract root dir
    if let Some(mut x) = matches.values_of("directory") {
        { assert_eq!(x.len(), 1); }
        if let Some(s) = x.next() {
            root = PathBuf::from_str(s).unwrap();
        } else {
            println!("DEPTH_LEVEL must be a integer number eg. 3");
        }
    }
// depth
    if let Some(mut nest) = matches.values_of("nest") {
        if let Some(s) = nest.next() {
            depth = u32::from_str(s).unwrap_or(1u32);
        } else {
            println!("PERCENT_THRESHOLD must be a integer number between 0~100 eg. 10");
        }
    }
// threshold
    if let Some(mut percent) = matches.values_of("percent") {
        if let Some(s) = percent.next() {
            threshold = u32::from_str(s).unwrap_or(0u32);
        }
    }
    (root, depth, threshold)
}

fn main() {
    // get args passed in
    let (root, depth, threshold) = parse_args();

    // calculating
    println!("Calcluate folder size of {:?} ...", root);
    let dirstat = calculate_dir_stat(0,root);

    // display result
    match dirstat {
        Some(ds) => {
            display_stat(&ds, depth, threshold, 0);
        },
        None => { }
    }
}

