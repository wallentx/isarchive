use isarchive::analyze;
use std::env;
use std::path::Path;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut files = Vec::new();
    let mut brief = false;
    let mut mime = false;

    if args.len() < 2 {
        print_usage_brief(&args[0]);
        process::exit(1);
    }

    for arg in args.iter().skip(1) {
        if arg.starts_with("--") {
            match arg.as_str() {
                "--brief" => brief = true,
                "--mime" => mime = true,
                "--help" => {
                    print_usage_detailed(&args[0]);
                    process::exit(0);
                }
                _ => {
                    eprintln!("Unknown option: {}", arg);
                    process::exit(1);
                }
            }
        } else if arg.starts_with("-") && arg.len() > 1 {
            for char in arg.chars().skip(1) {
                match char {
                    'b' => brief = true,
                    'i' => mime = true,
                    'h' => {
                        print_usage_brief(&args[0]);
                        process::exit(0);
                    }
                    _ => {
                        eprintln!("Unknown option: -{}", char);
                        process::exit(1);
                    }
                }
            }
        } else {
            files.push(arg);
        }
    }

    if files.is_empty() {
        print_usage_brief(&args[0]);
        process::exit(1);
    }

    let mut exit_code = 0;

    for path_str in files {
        let path = Path::new(path_str);
        if !path.exists() {
            eprintln!("{}: No such file or directory", path_str);
            exit_code = 2;
            continue;
        }

        let result = analyze(path);

        match result {
            Some(info) => {
                if brief {
                    println!("{}", info.category);
                } else if mime {
                    println!("{}: {}", path_str, info.category);
                } else {
                    let prefix = format!("{}: ", path_str);
                    if !info.hexdump.is_empty() {
                        println!(
                            "{}{}\nHex: {}\nType: {}",
                            prefix, info.description, info.hexdump, info.category
                        );
                    } else {
                        println!("{}{}\nType: {}", prefix, info.description, info.category);
                    }
                }
            }
            None => {
                if brief {
                    println!("not an archive");
                } else {
                    println!("{}: not an archive", path_str);
                }
                if exit_code == 0 {
                    exit_code = 1;
                }
            }
        }
    }

    process::exit(exit_code);
}

fn print_usage_brief(prog_name: &str) {
    println!("Usage: {} [-b] [-i] <file_path>...", prog_name);
}

fn print_usage_detailed(prog_name: &str) {
    println!("Usage: {} [OPTIONS] <file_path>...", prog_name);
    println!();
    println!("Detect archive types based on magic signatures and extensions.");
    println!();
    println!("Options:");
    println!("  -b, --brief    Print brief output (category only, no filename).");
    println!("  -i, --mime     Print MIME type/category (e.g., archive/storage).");
    println!("  -h             Print brief usage.");
    println!("  --help         Print this detailed help message.");
}
