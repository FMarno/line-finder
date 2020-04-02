use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("usage:\nline_ending.exe <fileorfoldername>");
        return;
    }
    let mut print_lf = false;
    if args.contains(&String::from("--lf")) {
        print_lf = true;
    }
    let mut print_crlf = false;
    if args.contains(&String::from("--crlf")) {
        print_crlf = true;
    }
    let mut print_mixed = false;
    if args.contains(&String::from("--mixed")) {
        print_mixed = true;
    }
    let target = Path::new(&args[1]);

    let mut paths: Vec<String> = Vec::new();
    if target.is_file() {
        paths.push(target.to_str().unwrap().to_owned());
    } else if target.is_dir() {
        paths.extend(
            target
                .read_dir()
                .unwrap()
                .filter_map(|path| path.map(|p| p.path().to_str().unwrap().to_owned()).ok()),
        );
    } else {
        println!("target is not a file or directory");
        return;
    }
    check_paths(paths, (print_lf, print_crlf, print_mixed))
}

fn check_paths(mut paths: Vec<String>, (print_lf, print_crlf, print_mixed): (bool, bool, bool)) {
    let mut lf_count = 0;
    let mut crlf_count = 0;
    let mut mixed_count = 0;
    let mut whitespace_count = 0;
    let mut files_checked = 0;

    while let Some(s) = paths.pop() {
        let target = Path::new(&s);
        if target.is_dir() {
            paths.extend(
                target
                    .read_dir()
                    .unwrap()
                    .filter_map(|path| path.map(|p| p.path().to_str().unwrap().to_owned()).ok()),
            );
        } else if target.is_file()
            && target
                .extension()
                .and_then(|ext| ext.to_str())
                .map_or(false, |ext| ext == "cs")
        {
            files_checked += 1;
            let ending = read_line_endings(target);
            match ending {
                Ok((LineEnding::LF, n)) => {
                    lf_count += 1;
                    whitespace_count += if n == 0 {
                        println!("{}", target.to_str().unwrap());
                        0
                    } else {
                        1
                    };
                    if print_lf {
                        println!("LF {}", target.to_str().unwrap());
                    }
                }
                Ok((LineEnding::CRLF, n)) => {
                    crlf_count += 1;
                    whitespace_count += if n == 0 {
                        println!("{}", target.to_str().unwrap());
                        0
                    } else {
                        1
                    };
                    if print_crlf {
                        println!("CRLF {}", target.to_str().unwrap());
                    }
                }
                Ok((LineEnding::Mixed, n)) => {
                    mixed_count += 1;
                    whitespace_count += if n == 0 {
                        println!("{}", target.to_str().unwrap());
                        0
                    } else {
                        1
                    };
                    if print_mixed {
                        println!("MIXED {}", target.to_str().unwrap());
                    }
                }
                _ => (),
            }
        }
    }

    println!("LF count {}", lf_count);
    println!("CRLF count {}", crlf_count);
    println!("MIXED count {}", mixed_count);
    println!("Files with bad line endings count {}", whitespace_count);
    println!("Files checked {}", files_checked)
}

#[derive(PartialEq, Debug)]
enum LineEnding {
    LF,
    CRLF,
    Mixed,
}

fn read_line_endings(path: &Path) -> std::io::Result<(LineEnding, u32)> {
    let f = File::open(path)?;
    let mut reader = BufReader::new(f);
    let mut endings = Vec::new();
    let mut buf = String::new();
    let mut whitespace = 0;
    while let Ok(n) = reader.read_line(&mut buf) {
        if n == 0 {
            break;
        }
        if buf.len() > buf.trim_end().len() {
            whitespace += 1;
        }
        let bytes = buf.as_bytes();
        if n == 1 && bytes[0] == '\n' as u8 {
            endings.push(LineEnding::LF);
            continue;
        }
        if n == 1 {
            continue;
        }
        match bytes[n - 2..] {
            [b'\r', b'\n'] => endings.push(LineEnding::CRLF),
            [_, b'\n'] => endings.push(LineEnding::LF),
            _ => (),
        }
    }
    let ending = if endings.iter().all(|ending| *ending == LineEnding::CRLF) {
        LineEnding::CRLF
    } else if endings.into_iter().all(|ending| ending == LineEnding::LF) {
        LineEnding::LF
    } else {
        LineEnding::Mixed
    };
    Ok((ending, whitespace))
}
