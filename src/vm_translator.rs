use std::collections::HashMap;
use std::fs::{read_dir, read_to_string};
use std::io::Write;
use std::path::PathBuf;

use once_cell::sync::OnceCell;

use crate::utils::{save_file, substr};

pub static ARITH_TABLE: OnceCell<HashMap<&str, &str>> = OnceCell::new();
pub static SEGMENT_TABLE: OnceCell<HashMap<&str, &str>> = OnceCell::new();

pub struct VmTranslator {
    vm_files:     Vec<PathBuf>,
    dest_path:    PathBuf,
    output:       Vec<u8>,
    symbol_index: u32,
    return_index: u32,
    multi_files:  bool,
}

impl VmTranslator {
    pub fn new(mut path: PathBuf) -> Self {
        ARITH_TABLE
            .set(HashMap::from([
                ("not", "!"),
                ("neg", "-"),
                ("add", "+"),
                ("sub", "-"),
                ("and", "&"),
                ("or", "|"),
                ("eq", "JNE"),
                ("lt", "JLE"),
                ("gt", "JGE"),
            ]))
            .unwrap();

        SEGMENT_TABLE
            .set(HashMap::from([
                ("local", "LCL"),
                ("argument", "ARG"),
                ("this", "THIS"),
                ("that", "THAT"),
                ("temp", "5"),
                ("pointer", "3"),
            ]))
            .unwrap();

        let mut multi_files = false;
        let mut vm_files = vec![];
        #[allow(unused_assignments)]
        let mut dest_path = PathBuf::default();

        if path.is_dir() {
            multi_files = true;
            for file in read_dir(path.clone()).unwrap() {
                let file = file.unwrap();
                let path = file.path();

                if path.extension().unwrap() == "vm" {
                    vm_files.push(path);
                }
            }
            path.push(format!(
                "{}.asm",
                path.file_name().unwrap().to_str().unwrap()
            ));
            dest_path = path;
        } else {
            assert_eq!(path.extension().unwrap(), "vm");
            vm_files.push(path.clone());
            dest_path = {
                path.set_extension("asm");
                path
            };
        }

        Self {
            vm_files,
            dest_path,
            output: vec![],
            symbol_index: 0,
            return_index: 0,
            multi_files,
        }
    }

    pub fn dest_path(&self) -> &PathBuf {
        &self.dest_path
    }

    pub fn run(&mut self) {
        if self.multi_files {
            // add bootstrap codes
            // SP = 256
            writeln!(&mut self.output, "@256").unwrap();
            writeln!(&mut self.output, "D=A").unwrap();
            writeln!(&mut self.output, "@SP").unwrap();
            writeln!(&mut self.output, "M=D").unwrap();

            for (i, seg) in vec!["LCL", "ARG", "THIS", "THAT"].into_iter().enumerate() {
                writeln!(&mut self.output, "@{}", i + 1).unwrap();
                writeln!(&mut self.output, "D=A").unwrap();
                writeln!(&mut self.output, "@{}", seg).unwrap();
                writeln!(&mut self.output, "M=D").unwrap();
            }

            // push retAddr
            writeln!(&mut self.output, "@bootstrap").unwrap();
            writeln!(&mut self.output, "D=A").unwrap();
            push_d(&mut self.output);

            // push LCL, ARG, THIS, THAT
            for seg in &["LCL", "ARG", "THIS", "THAT"] {
                writeln!(&mut self.output, "@{}", seg).unwrap();
                writeln!(&mut self.output, "D=M").unwrap();
                push_d(&mut self.output);
            }

            // ARG = SP - n - 5
            writeln!(&mut self.output, "@5").unwrap();
            writeln!(&mut self.output, "D=A").unwrap();
            writeln!(&mut self.output, "@SP").unwrap();
            writeln!(&mut self.output, "D=M-D").unwrap();
            writeln!(&mut self.output, "@ARG").unwrap();
            writeln!(&mut self.output, "M=D").unwrap();

            // LCL = SP
            writeln!(&mut self.output, "@SP").unwrap();
            writeln!(&mut self.output, "D=M").unwrap();
            writeln!(&mut self.output, "@LCL").unwrap();
            writeln!(&mut self.output, "M=D").unwrap();

            writeln!(&mut self.output, "@Sys.init").unwrap();
            writeln!(&mut self.output, "0;JMP").unwrap();
            writeln!(&mut self.output, "(bootstrap)").unwrap();
        }

        for file in self.vm_files.iter() {
            let mut vm =
                SingleVmTranslator::new(file.to_owned(), self.symbol_index, self.return_index);
            vm.parse();

            self.output.extend(vm.output);
            self.symbol_index = vm.symbol_index;
            self.return_index = vm.return_index;
        }
    }

    pub fn save_file(&self) {
        save_file(&self.output, &self.dest_path).unwrap();
    }
}

pub struct SingleVmTranslator {
    vm_filename:  String,
    codes:        Vec<String>,
    output:       Vec<u8>,
    symbol_index: u32,
    return_index: u32,
    current_func: String,
}

impl SingleVmTranslator {
    pub fn new(path: PathBuf, symbol_index: u32, return_index: u32) -> Self {
        assert_eq!(path.extension().unwrap(), "vm");

        let vm_filename = path.file_stem().unwrap().to_str().unwrap().to_owned();
        let mut codes = vec![];

        for line in read_to_string(path).unwrap().lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('/') {
                continue;
            }
            if line.find('/').is_some() {
                codes.push(substr(line, 0, line.find('/').unwrap()).trim().to_string());
                continue;
            }
            codes.push(line.to_string());
        }

        Self {
            vm_filename,
            codes,
            output: vec![],
            symbol_index,
            return_index,
            current_func: "".to_owned(),
        }
    }

    // For each line in self.codes, generate its corresponding assembly codes
    pub fn parse(&mut self) {
        let codes = self.codes.clone();

        for code in codes.iter() {
            let parts: Vec<&str> = code.split(' ').collect();

            match parts[0] {
                "push" => self.parse_c_push(parts),
                "pop" => self.parse_c_pop(parts),
                "label" => self.parse_c_label(parts),
                "goto" => self.parse_c_goto(parts),
                "if-goto" => self.parse_c_if_goto(parts),
                "function" => self.parse_c_function(parts),
                "call" => self.parse_c_call(parts),
                "return" => self.parse_c_return(),
                _ => self.parse_c_arithmetic(parts[0]),
            }
        }
    }

    fn parse_c_arithmetic(&mut self, command: &str) {
        match command {
            "not" | "neg" => {
                writeln!(&mut self.output, "@SP").unwrap();
                writeln!(&mut self.output, "A=M-1").unwrap();
                writeln!(
                    &mut self.output,
                    "M={}M\n",
                    ARITH_TABLE.get().unwrap().get(command).unwrap()
                )
                .unwrap();
            }
            "add" | "sub" | "and" | "or" => {
                writeln!(&mut self.output, "@SP").unwrap();
                writeln!(&mut self.output, "AM=M-1").unwrap();
                writeln!(&mut self.output, "D=M").unwrap();
                writeln!(&mut self.output, "A=A-1").unwrap();
                writeln!(
                    &mut self.output,
                    "M=M{}D",
                    ARITH_TABLE.get().unwrap().get(command).unwrap()
                )
                .unwrap();
            }
            "eq" | "gt" | "lt" => {
                writeln!(&mut self.output, "@SP").unwrap();
                writeln!(&mut self.output, "AM=M-1").unwrap();
                writeln!(&mut self.output, "D=M").unwrap();
                writeln!(&mut self.output, "A=A-1").unwrap();
                writeln!(&mut self.output, "D=M-D").unwrap();
                writeln!(&mut self.output, "M=0").unwrap();
                writeln!(&mut self.output, "@{}_{}", command, self.symbol_index).unwrap();
                writeln!(
                    &mut self.output,
                    "D;{}",
                    ARITH_TABLE.get().unwrap().get(command).unwrap()
                )
                .unwrap();
                writeln!(&mut self.output, "@SP").unwrap();
                writeln!(&mut self.output, "A=M-1").unwrap();
                writeln!(&mut self.output, "M=-1").unwrap();
                writeln!(&mut self.output, "({}_{})", command, self.symbol_index).unwrap();
                self.symbol_index += 1;
            }
            _ => unreachable!("invalid command: arithmetic"),
        }
    }

    fn parse_c_push(&mut self, command: Vec<&str>) {
        match command[1] {
            "constant" => {
                writeln!(&mut self.output, "@{}", command[2]).unwrap();
                writeln!(&mut self.output, "D=A").unwrap();
            }
            "local" | "argument" | "this" | "that" => {
                writeln!(&mut self.output, "@{}", command[2]).unwrap();
                writeln!(&mut self.output, "D=A").unwrap();
                writeln!(
                    &mut self.output,
                    "@{}",
                    SEGMENT_TABLE.get().unwrap().get(command[1]).unwrap()
                )
                .unwrap();
                writeln!(&mut self.output, "A=M+D").unwrap();
                writeln!(&mut self.output, "D=M").unwrap();
            }
            "temp" | "pointer" => {
                writeln!(&mut self.output, "@{}", command[2]).unwrap();
                writeln!(&mut self.output, "D=A").unwrap();
                writeln!(
                    &mut self.output,
                    "@{}",
                    SEGMENT_TABLE.get().unwrap().get(command[1]).unwrap()
                )
                .unwrap();
                writeln!(&mut self.output, "A=A+D").unwrap();
                writeln!(&mut self.output, "D=M").unwrap();
            }
            "static" => {
                writeln!(&mut self.output, "@{}.{}", self.vm_filename, command[2]).unwrap();
                writeln!(&mut self.output, "D=M").unwrap();
            }
            _ => unreachable!("invalid command: push"),
        }
        push_d(&mut self.output);
    }

    fn parse_c_pop(&mut self, command: Vec<&str>) {
        match command[1] {
            "local" | "argument" | "this" | "that" => {
                writeln!(&mut self.output, "@{}", command[2]).unwrap();
                writeln!(&mut self.output, "D=A").unwrap();
                writeln!(
                    &mut self.output,
                    "@{}",
                    SEGMENT_TABLE.get().unwrap().get(command[1]).unwrap()
                )
                .unwrap();
                writeln!(&mut self.output, "D=M+D").unwrap();
                writeln!(&mut self.output, "@R15").unwrap();
                writeln!(&mut self.output, "M=D").unwrap();
            }
            "temp" | "pointer" => {
                writeln!(&mut self.output, "@{}", command[2]).unwrap();
                writeln!(&mut self.output, "D=A").unwrap();
                writeln!(
                    &mut self.output,
                    "@{}",
                    SEGMENT_TABLE.get().unwrap().get(command[1]).unwrap()
                )
                .unwrap();
                writeln!(&mut self.output, "D=A+D").unwrap();
                writeln!(&mut self.output, "@R15").unwrap();
                writeln!(&mut self.output, "M=D").unwrap();
            }
            "static" => {
                writeln!(&mut self.output, "@{}.{}", self.vm_filename, command[2]).unwrap();
                writeln!(&mut self.output, "D=A").unwrap();
                writeln!(&mut self.output, "R15").unwrap();
                writeln!(&mut self.output, "M=D").unwrap();
            }
            _ => unreachable!("invalid command: pop"),
        }

        // SP--, then put the value *SP into M[address]
        writeln!(&mut self.output, "@SP").unwrap();
        writeln!(&mut self.output, "AM=M-1").unwrap();
        writeln!(&mut self.output, "D=M").unwrap();
        writeln!(&mut self.output, "@R15").unwrap();
        writeln!(&mut self.output, "A=M").unwrap();
        writeln!(&mut self.output, "M=D").unwrap();
    }

    fn parse_c_label(&mut self, command: Vec<&str>) {
        writeln!(&mut self.output, "({}${})", self.current_func, command[1]).unwrap();
    }

    fn parse_c_goto(&mut self, command: Vec<&str>) {
        writeln!(&mut self.output, "@{}${}", self.current_func, command[1]).unwrap();
        writeln!(&mut self.output, "0;JMP").unwrap();
    }

    fn parse_c_if_goto(&mut self, command: Vec<&str>) {
        writeln!(&mut self.output, "@SP").unwrap();
        writeln!(&mut self.output, "AM=M-1").unwrap();
        writeln!(&mut self.output, "D=M").unwrap();
        writeln!(&mut self.output, "@{}${}", self.current_func, command[1]).unwrap();
        writeln!(&mut self.output, "D;JNE").unwrap();
    }

    fn parse_c_function(&mut self, command: Vec<&str>) {
        self.current_func = command[1].to_owned();

        writeln!(&mut self.output, "({})", command[1]).unwrap();

        for _ in 0..command[2].parse::<u32>().unwrap() {
            writeln!(&mut self.output, "@SP").unwrap();
            writeln!(&mut self.output, "A=M").unwrap();
            writeln!(&mut self.output, "M=0").unwrap();
            writeln!(&mut self.output, "@SP").unwrap();
            writeln!(&mut self.output, "M=M+1").unwrap();
        }
    }

    fn parse_c_call(&mut self, command: Vec<&str>) {
        let label = format!("End${}${}", command[1], self.return_index);

        // push return address
        writeln!(&mut self.output, "@{}", label).unwrap();
        writeln!(&mut self.output, "D=A").unwrap();
        push_d(&mut self.output);

        // push LCL, ARG, THIS, THAT
        for segment in &["LCL", "ARG", "THIS", "THAT"] {
            writeln!(&mut self.output, "@{}", segment).unwrap();
            writeln!(&mut self.output, "D=M").unwrap();
            push_d(&mut self.output);
        }

        // ARG = SP - n - 5, n is the count of paramers
        writeln!(
            &mut self.output,
            "@{}",
            command[2].parse::<u32>().unwrap() + 5
        )
        .unwrap();
        writeln!(&mut self.output, "D=A").unwrap();
        writeln!(&mut self.output, "@SP").unwrap();
        writeln!(&mut self.output, "D=M-D").unwrap();
        writeln!(&mut self.output, "@ARG").unwrap();
        writeln!(&mut self.output, "M=D").unwrap();

        // LCL = SP
        writeln!(&mut self.output, "@SP").unwrap();
        writeln!(&mut self.output, "D=M").unwrap();
        writeln!(&mut self.output, "@LCL").unwrap();
        writeln!(&mut self.output, "M=D").unwrap();

        // goto f
        writeln!(&mut self.output, "@{}", command[1]).unwrap();
        writeln!(&mut self.output, "0;JMP").unwrap();

        // return label
        writeln!(&mut self.output, "({})", label).unwrap();

        self.return_index += 1;
    }

    fn parse_c_return(&mut self) {
        // FRAME = LCL, put LCL into R15
        writeln!(&mut self.output, "@LCL").unwrap();
        writeln!(&mut self.output, "D=M").unwrap();
        writeln!(&mut self.output, "@R15").unwrap();
        writeln!(&mut self.output, "M=D").unwrap();

        // RET = *(FRAME - 5), put retAddr into R14
        writeln!(&mut self.output, "@5").unwrap();
        writeln!(&mut self.output, "D=A").unwrap();
        writeln!(&mut self.output, "@R15").unwrap();
        writeln!(&mut self.output, "A=M-D").unwrap();
        writeln!(&mut self.output, "D=M").unwrap();
        writeln!(&mut self.output, "@R14").unwrap();
        writeln!(&mut self.output, "M=D").unwrap();

        // *ARG = pop()
        writeln!(&mut self.output, "@SP").unwrap();
        writeln!(&mut self.output, "AM=M-1").unwrap();
        writeln!(&mut self.output, "D=M").unwrap();
        writeln!(&mut self.output, "@ARG").unwrap();
        writeln!(&mut self.output, "A=M").unwrap();
        writeln!(&mut self.output, "M=D").unwrap();

        // SP = ARG + 1
        writeln!(&mut self.output, "@ARG").unwrap();
        writeln!(&mut self.output, "D=M+1").unwrap();
        writeln!(&mut self.output, "@SP").unwrap();
        writeln!(&mut self.output, "M=D").unwrap();

        // THAT = *(FRAME - 1)
        writeln!(&mut self.output, "@R15").unwrap();
        writeln!(&mut self.output, "A=M-1").unwrap();
        writeln!(&mut self.output, "D=M").unwrap();
        writeln!(&mut self.output, "@THAT").unwrap();
        writeln!(&mut self.output, "M=D").unwrap();

        // THIS = *(FRAME - 2)
        // ARG = *(FRAME - 3)
        // LCL = *(FRAME - 4)
        for (i, seg) in vec!["THIS", "ARG", "LCL"].into_iter().enumerate() {
            writeln!(&mut self.output, "@{}", i + 2).unwrap();
            writeln!(&mut self.output, "D=A").unwrap();
            writeln!(&mut self.output, "@R15").unwrap();
            writeln!(&mut self.output, "A=M-D").unwrap();
            writeln!(&mut self.output, "D=M").unwrap();
            writeln!(&mut self.output, "@{}", seg).unwrap();
            writeln!(&mut self.output, "M=D").unwrap();
        }

        // goto RET
        writeln!(&mut self.output, "@R14").unwrap();
        writeln!(&mut self.output, "A=M").unwrap();
        writeln!(&mut self.output, "0;JMP").unwrap();
    }
}

fn push_d(output: &mut Vec<u8>) {
    writeln!(output, "@SP").unwrap();
    writeln!(output, "A=M").unwrap();
    writeln!(output, "M=D").unwrap();
    writeln!(output, "@SP").unwrap();
    writeln!(output, "M=M+1").unwrap();
}
