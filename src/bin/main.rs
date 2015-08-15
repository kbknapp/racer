#![cfg_attr(all(test, feature = "nightly"), feature(test))] // we only need test feature when testing

#[macro_use] extern crate log;

extern crate syntex_syntax;
extern crate toml;
extern crate env_logger;
extern crate rustc_serialize;
extern crate docopt;

extern crate racer;

#[cfg(not(test))]
use racer::core;
#[cfg(not(test))]
use racer::util;
#[cfg(not(test))]
use racer::core::Match;
#[cfg(not(test))]
use racer::util::{getline, path_exists};
#[cfg(not(test))]
use racer::nameres::{do_file_search, do_external_search, PATH_SEP};
#[cfg(not(test))]
use racer::scopes;
#[cfg(not(test))]
use std::path::Path;


#[cfg(not(test))]
fn match_with_snippet_fn(m: Match) {
    let (linenum, charnum) = scopes::point_to_coords_from_file(&m.filepath, m.point).unwrap();
    if m.matchstr == "" {
        panic!("MATCHSTR is empty - waddup?");
    }

    let snippet = racer::snippets::snippet_for_match(&m);
    println!("MATCH {};{};{};{};{};{:?};{}", m.matchstr,
                                    snippet,
                                    linenum.to_string(),
                                    charnum.to_string(),
                                    m.filepath.to_str().unwrap(),
                                    m.mtype,
                                    m.contextstr
             );
}

#[cfg(not(test))]
fn match_fn(m: Match) {
    let (linenum, charnum) = scopes::point_to_coords_from_file(&m.filepath, m.point).unwrap();
    println!("MATCH {},{},{},{},{:?},{}", m.matchstr,
                                    linenum.to_string(),
                                    charnum.to_string(),
                                    m.filepath.to_str().unwrap(),
                                    m.mtype,
                                    m.contextstr
             );
}

#[cfg(not(test))]
fn complete(match_found: &Fn(Match), args: &Args) {
    if args.arg_fully_qualified_name.is_empty() {
        // input: linenum, colnum, fname
        let substitute_file = Path::new(if let Some(ref p) = args.flag_substitute_file {
            &*p
        }else {
            &args.arg_fname
        });
        let fpath = Path::new(&*args.arg_fname);
        let src = core::load_file(&substitute_file);
        let line = &*getline(&substitute_file, args.arg_linenum);
        let (start, pos) = util::expand_ident(line, args.arg_charnum);
        println!("PREFIX {},{},{}", start, pos, &line[start..pos]);

        let session = core::Session::from_path(&fpath, &substitute_file);
        let point = scopes::coords_to_point(&*src, args.arg_linenum, args.arg_charnum);
        for m in core::complete_from_file(&*src, &fpath, point, &session) {
            match_found(m);
        }
        println!("END");
    } else {
        // input: a command line string passed in
        let it = args.arg_fully_qualified_name.split("::");
        let p: Vec<&str> = it.collect();

        for m in do_file_search(p[0], &Path::new(".")) {
            if p.len() == 1 {
                match_found(m);
            } else {
                for m in do_external_search(&p[1..], &m.filepath, m.point, core::SearchType::StartsWith, core::Namespace::BothNamespaces, &m.session) {
                    match_found(m);
                }
            }
        }
    }
}

#[cfg(not(test))]
fn prefix(args: &Args) {
    // print the start, end, and the identifier prefix being matched
    let path = Path::new(&*args.arg_fname);
    let line = &*getline(&path, args.arg_linenum);
    let (start, pos) = util::expand_ident(line, args.arg_charnum);
    println!("PREFIX {},{},{}", start, pos, &line[start..pos]);
}

#[cfg(not(test))]
fn find_definition(args: &Args) {
    let substitute_file = Path::new(if let Some(ref p) = args.flag_substitute_file {
        &*p
    }else {
        &args.arg_fname
    });
    let fpath = Path::new(&args.arg_fname);
    let session = core::Session::from_path(&fpath, &substitute_file);
    let src = core::load_file(&substitute_file);
    let pos = scopes::coords_to_point(&*src, args.arg_linenum, args.arg_charnum);

    core::find_definition(&*src, &fpath, pos, &session).map(match_fn);
    println!("END");
}

#[cfg(not(test))]
fn check_rust_src_env_var() {
    if let Ok(srcpaths) = std::env::var("RUST_SRC_PATH") {
        let v = srcpaths.split(PATH_SEP).collect::<Vec<_>>();
        if !v.is_empty() {
            let f = Path::new(v[0]);
            if !path_exists(f) {
                println!("racer can't find the directory pointed to by the RUST_SRC_PATH variable \"{}\". Try using an absolute fully qualified path and make sure it points to the src directory of a rust checkout - e.g. \"/home/foouser/src/rust/src\".", srcpaths);
                std::process::exit(1);
            } else if !path_exists(f.join("libstd")) {
                println!("Unable to find libstd under RUST_SRC_PATH. N.B. RUST_SRC_PATH variable needs to point to the *src* directory inside a rust checkout e.g. \"/home/foouser/src/rust/src\". Current value \"{}\"", srcpaths);
                std::process::exit(1);
            }
        }
    } else {
        println!("RUST_SRC_PATH environment variable must be set to point to the src directory of a rust checkout. E.g. \"/home/foouser/src/rust/src\"");
        std::process::exit(1);
    }
}

#[cfg(not(test))]
fn daemon() {
    use std::io;
    let mut input = String::new();
    while let Ok(n) = io::stdin().read_line(&mut input) {
        if n == 0 {
            break;
        }
        let args: Vec<String> = input.split(" ").map(|s| s.trim().to_string()).collect();

        let a: Args = docopt::Docopt::new(USAGE)
            .and_then(|d| Ok(d.argv(args.iter())))
            .and_then(|d| d.decode())
            .unwrap_or_else(|e| e.exit());


        run(a);
        input.clear();
    }
}

static USAGE: &'static str = "
Rust code completion utility

Usage:
  racer daemon
  racer complete <linenum> <charnum> <fname> [--substitute-file]
  racer complete-with-snippet <linenum> <charnum> <fname> [--substitute-file]
  racer find-definition <linenum> <charnum> <fname> [--substitute-file]
  racer complete <fully-qualified-name>
  racer complete-with-snippet <fully-qualified-name>
  racer prefix <linenum> <charnum> <fname>
  racer (-h | --help)
  racer --version

Options:
  --substitute-file
  -h, --help         Show this screen.
  --version          Show version.

Commands:
  daemon                : Start a process that receives the above commands via stdin
  prefix                :
  find_definition       :
  complete              : Provide a completion
  complete_with_snippet : Provide a more detailed completion
";

#[cfg(not(test))]
#[derive(Debug, RustcDecodable)]
struct Args {
    cmd_daemon: bool,

    cmd_complete: bool,
    arg_linenum: usize,
    arg_charnum: usize,
    arg_fname: String,
    flag_substitute_file: Option<String>,

    cmd_find_definition: bool,

    arg_fully_qualified_name: String,

    cmd_prefix: bool,

    cmd_complete_with_snippet: bool,
}

#[cfg(not(test))]
fn print_usage() {
    println!("{}", USAGE);
}

#[cfg(not(test))]
fn main() {
    env_logger::init().unwrap();
    check_rust_src_env_var();

    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        print_usage();
        std::process::exit(1);
    }

    let a: Args = docopt::Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    run(a);
}

#[cfg(not(test))]
fn run(args: Args) {
    if args.cmd_daemon {
        daemon()
    } else if args.cmd_prefix {
        prefix(&args)
    } else if args.cmd_complete {
        complete(&match_fn, &args)
    } else if args.cmd_complete_with_snippet {
        complete(&match_with_snippet_fn, &args)
    } else if args.cmd_find_definition {
        find_definition(&args)
    } else {
        println!("Sorry, I didn't understand that command");
        print_usage();
        std::process::exit(1);
    }
}
