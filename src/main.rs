use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("usage:\nline_ending.exe <fileorfoldername>");
        return;
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

    while let Some(s) = paths.pop() {
        let target = Path::new(&s);
        print!("{} ", s);
        if target.is_dir() {
            paths.extend(
                target
                    .read_dir()
                    .unwrap()
                    .filter_map(|path| path.map(|p| p.path().to_str().unwrap().to_owned()).ok()),
            );
            println!();
        } else if target.is_file() {
            let ending = read_line_endings(target);
            println!("{:?}", ending);
        }
    }
}

#[derive(PartialEq, Debug)]
enum LineEnding {
    LF,
    CRLF,
    Mixed,
}

fn read_line_endings(path: &Path) -> std::io::Result<LineEnding> {
    let f = File::open(path)?;
    let mut reader = BufReader::new(f);
    let mut endings = Vec::new();
    let mut buf = String::new();
    while let Ok(n) = reader.read_line(&mut buf){
        if n == 0{break;}
        let bytes = buf.as_bytes();
        if n == 1 && bytes[0] == '\n' as u8 {
            endings.push(LineEnding::LF);
        }
        match bytes[bytes.len()-2..] {
            [b'\r', b'\n'] => endings.push(LineEnding::CRLF),
            [_, b'\n'] => endings.push(LineEnding::LF),
            _ => (),
        }
    }
    if endings.iter().all(|ending| *ending == LineEnding::CRLF) {
        Ok(LineEnding::CRLF)
    } else if endings.into_iter().all(|ending| ending == LineEnding::LF) {
        Ok(LineEnding::LF)
    } else {
        Ok(LineEnding::Mixed)
    }
}
