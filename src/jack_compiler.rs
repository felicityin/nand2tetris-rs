use std::path::PathBuf;

use crate::jack_parser::*;
use crate::jack_tokenizer::JackTokenizer;
use crate::vm_writer::VmWriter;

pub fn compile_to_vm(mut path: PathBuf) {
    let mut tokenizer = JackTokenizer::new(path.clone());
    tokenizer.run();

    let mut parser = JackParser::new(tokenizer.tokens());
    parser.run();

    let mut vm_writer = VmWriter::new(parser.ast());
    vm_writer.run();

    path.set_extension("vm");
    vm_writer.save_file(path);
}
