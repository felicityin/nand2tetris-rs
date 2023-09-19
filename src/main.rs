mod assembler;
mod jack_analyzer;
mod jack_tokenizer;
mod utils;
mod vm_translator;

use std::ffi::OsString;
use std::path::PathBuf;

use assembler::Assembler;
use jack_analyzer::JackAnalyzer;
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
            clap::Command::new("analyze")
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
        );

    let matches = cmd.get_matches();

    match matches.subcommand() {
        Some(("asm", matches)) => assembly(matches),
        Some(("vm", matches)) => vm_translate(matches),
        Some(("token", matches)) => tokenize(matches),
        Some(("analyze", matches)) => analyze(matches),
        _ => unreachable!(),
    }
}

fn assembly(matches: &clap::ArgMatches) {
    let path = matches.get_one::<OsString>("path").unwrap();
    let path = PathBuf::from(path).canonicalize().unwrap();
    let mut assembler = Assembler::new(path);
    assembler.run();
    println!("\noutput: {}", assembler.dest_path().to_str().unwrap());
}

fn vm_translate(matches: &clap::ArgMatches) {
    let path = matches.get_one::<OsString>("path").unwrap();
    let path = PathBuf::from(path).canonicalize().unwrap();
    let mut vm_translator = VmTranslator::new(path);
    vm_translator.run();
    println!("\noutput: {}", vm_translator.dest_path().to_str().unwrap());
}

fn tokenize(matches: &clap::ArgMatches) {
    let path = matches.get_one::<OsString>("path").unwrap();
    let path = PathBuf::from(path).canonicalize().unwrap();
    let mut tokenizer = JackTokenizer::new(path);
    tokenizer.tokenize();
    tokenizer.save_file();
    println!("\noutput: {}", tokenizer.dest_path().to_str().unwrap());
}

fn analyze(matches: &clap::ArgMatches) {
    let path = matches.get_one::<OsString>("path").unwrap();
    let path = PathBuf::from(path).canonicalize().unwrap();

    let mut tokenizer = JackTokenizer::new(path.clone());
    tokenizer.tokenize();

    let mut analyzer = JackAnalyzer::new(path, tokenizer.tokens());
    analyzer.analyze();
    analyzer.save_file();
    println!("\noutput: {}", analyzer.dest_path().to_str().unwrap());
}
