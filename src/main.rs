mod assembler;
mod ast;
mod jack_compiler;
mod jack_parser;
mod jack_tokenizer;
mod symbol_table;
mod utils;
mod vm_translator;
mod vm_writer;

use std::ffi::OsString;
use std::path::PathBuf;

use assembler::Assembler;
use jack_compiler::compile_to_vm;
use jack_parser::JackParser;
use jack_tokenizer::JackTokenizer;
use vm_translator::VmTranslator;

fn main() {
    let cmd = clap::Command::new("compiler")
        .version(clap::crate_version!())
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand_required(true)
        .subcommand(
            clap::Command::new("asm")
                .about("Compile *.asm file into *.hack file")
                .arg(
                    clap::Arg::new("path")
                        .long("path")
                        .short('p')
                        .required(true)
                        .num_args(1)
                        .value_parser(clap::builder::ValueParser::os_string())
                        .help("path to *.asm file"),
                ),
        )
        .subcommand(
            clap::Command::new("vm")
                .about("Compile *.vm file into *.asm file")
                .arg(
                    clap::Arg::new("path")
                        .long("path")
                        .short('p')
                        .required(true)
                        .num_args(1)
                        .value_parser(clap::builder::ValueParser::os_string())
                        .help("path to *.vm file or directory"),
                ),
        )
        .subcommand(
            clap::Command::new("token")
                .about("Compile *.jack file into *.token.xml file")
                .arg(
                    clap::Arg::new("path")
                        .long("path")
                        .short('p')
                        .required(true)
                        .num_args(1)
                        .value_parser(clap::builder::ValueParser::os_string())
                        .help("path to *.jack file"),
                ),
        )
        .subcommand(
            clap::Command::new("parse")
                .about("Compile *.jack file into *.tree.xml file")
                .arg(
                    clap::Arg::new("path")
                        .long("path")
                        .short('p')
                        .required(true)
                        .num_args(1)
                        .value_parser(clap::builder::ValueParser::os_string())
                        .help("path to *.jack file"),
                ),
        )
        .subcommand(
            clap::Command::new("compile")
                .about("Compile *.jack file into *.vm file")
                .arg(
                    clap::Arg::new("path")
                        .long("path")
                        .short('p')
                        .required(true)
                        .num_args(1)
                        .value_parser(clap::builder::ValueParser::os_string())
                        .help("path to *.jack file"),
                ),
        );

    let matches = cmd.get_matches();

    match matches.subcommand() {
        Some(("asm", matches)) => assembly(matches),
        Some(("vm", matches)) => vm_translate(matches),
        Some(("token", matches)) => tokenize(matches),
        Some(("parse", matches)) => parse(matches),
        Some(("compile", matches)) => compile(matches),
        _ => unreachable!(),
    }
}

fn assembly(matches: &clap::ArgMatches) {
    let path = matches.get_one::<OsString>("path").unwrap();
    let path = PathBuf::from(path).canonicalize().unwrap();

    let mut assembler = Assembler::new(path.clone());
    assembler.run();

    let mut dst_path = PathBuf::from(path.parent().unwrap());
    dst_path.push(format!(
        "output/{}.vm",
        path.file_name().unwrap().to_str().unwrap()
    ));
    dst_path.set_extension("vm");
    println!("\noutput: {}", dst_path.to_str().unwrap());

    assembler.save_binary(&dst_path);
}

fn vm_translate(matches: &clap::ArgMatches) {
    let path = matches.get_one::<OsString>("path").unwrap();
    let path = PathBuf::from(path).canonicalize().unwrap();

    let mut vm_translator = VmTranslator::new(path.clone());
    vm_translator.run();

    let dst_path = if path.is_dir() {
        let mut dst_path = path.clone();
        dst_path.push(format!(
            "output/{}.asm",
            path.file_name().unwrap().to_str().unwrap()
        ));
        dst_path
    } else {
        let mut dst_path = PathBuf::from(path.parent().unwrap());
        dst_path.push(format!(
            "output/{}",
            path.file_name().unwrap().to_str().unwrap()
        ));
        dst_path.set_extension("asm");
        dst_path
    };
    println!("\noutput: {}", dst_path.to_str().unwrap());

    vm_translator.save_file(&dst_path);
}

fn tokenize(matches: &clap::ArgMatches) {
    let path = matches.get_one::<OsString>("path").unwrap();
    let path = PathBuf::from(path).canonicalize().unwrap();

    let mut tokenizer = JackTokenizer::new(path.clone());
    tokenizer.run();

    let mut dst_path = PathBuf::from(path.parent().unwrap());
    dst_path.push(format!(
        "output/{}",
        path.file_name().unwrap().to_str().unwrap()
    ));
    dst_path.set_extension("token.xml");
    println!("\noutput: {}", dst_path.to_str().unwrap());

    tokenizer.save_file(&dst_path);
}

fn parse(matches: &clap::ArgMatches) {
    let path = matches.get_one::<OsString>("path").unwrap();
    let path = PathBuf::from(path).canonicalize().unwrap();

    let mut tokenizer = JackTokenizer::new(path.clone());
    tokenizer.run();

    let mut parser = JackParser::new(tokenizer.tokens());
    parser.run();

    let mut dst_path = PathBuf::from(path.parent().unwrap());
    dst_path.push(format!(
        "output/{}",
        path.file_name().unwrap().to_str().unwrap()
    ));
    dst_path.set_extension("tree.xml");
    println!("\noutput: {}", dst_path.to_str().unwrap());

    parser.save_file(&dst_path);
}

fn compile(matches: &clap::ArgMatches) {
    let path = matches.get_one::<OsString>("path").unwrap();
    let path = PathBuf::from(path).canonicalize().unwrap();

    compile_to_vm(path);
}
