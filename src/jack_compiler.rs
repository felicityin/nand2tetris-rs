use std::path::PathBuf;

use crate::jack_parser::*;
use crate::jack_tokenizer::JackTokenizer;
use crate::vm_writer::VmWriter;

pub fn compile_to_vm(path: PathBuf) {
    let mut tokenizer = JackTokenizer::new(path.clone());
    tokenizer.run();

    let mut parser = JackParser::new(tokenizer.tokens());
    parser.run();

    let mut vm_writer = VmWriter::new(parser.ast());
    vm_writer.run();

    let mut dst_path = PathBuf::from(path.parent().unwrap());
    dst_path.push(format!(
        "output/{}",
        path.file_name().unwrap().to_str().unwrap()
    ));
    dst_path.set_extension("vm");
    println!("\noutput: {}", dst_path.to_str().unwrap());

    vm_writer.save_file(dst_path);
}
