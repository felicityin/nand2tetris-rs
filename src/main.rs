mod assembler;
mod jack_tokenizer;
mod vm_translator;

use std::ffi::OsString;
use std::path::PathBuf;

use assembler::Assembler;
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
        );

    let matches = cmd.get_matches();

    match matches.subcommand() {
        Some(("asm", matches)) => assembly(matches),
        Some(("vm", matches)) => vm_translate(matches),
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
