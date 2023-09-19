use std::collections::HashMap;
use std::fs::read_to_string;
use std::io::Write;
use std::path::PathBuf;

use once_cell::sync::OnceCell;

use crate::utils::{save_file, substr};

pub static COMP_TABLE: OnceCell<HashMap<&str, &str>> = OnceCell::new();
pub static DEST_TABLE: OnceCell<HashMap<&str, &str>> = OnceCell::new();
pub static JMP_TABLE: OnceCell<HashMap<&str, &str>> = OnceCell::new();
pub static PREDEFINED_SYMBOL_TABLE: OnceCell<HashMap<&str, u32>> = OnceCell::new();

pub struct Assembler {
    dest_path:     PathBuf,
    symbol_table:  HashMap<String, u32>,
    codes:         Vec<String>,
    output:        Vec<u8>,
    alloc_address: u32,
}

impl Assembler {
    /// Read in the file and ignore the blank lines and comment lines
    pub fn new(mut path: PathBuf) -> Self {
        COMP_TABLE
            .set(HashMap::from([
                ("0", "0101010"),
                ("1", "0111111"),
                ("-1", "0111010"),
                ("D", "0001100"),
                ("A", "0110000"),
                ("!D", "0001101"),
                ("!A", "0110001"),
                ("-D", "0001111"),
                ("-A", "0110011"),
                ("D+1", "0011111"),
                ("A+1", "0110111"),
                ("D-1", "0001110"),
                ("A-1", "0110010"),
                ("D+A", "0000010"),
                ("D-A", "0010011"),
                ("A-D", "0000111"),
                ("D&A", "0000000"),
                ("D|A", "0010101"),
                ("M", "1110000"),
                ("!M", "1110001"),
                ("-M", "1110011"),
                ("M+1", "1110111"),
                ("M-1", "1110010"),
                ("D+M", "1000010"),
                ("D-M", "1010011"),
                ("M-D", "1000111"),
                ("D&M", "1000000"),
                ("D|M", "1010101"),
            ]))
            .unwrap();

        DEST_TABLE
            .set(HashMap::from([
                ("null", "000"),
                ("M", "001"),
                ("D", "010"),
                ("MD", "011"),
                ("A", "100"),
                ("AM", "101"),
                ("AD", "110"),
                ("AMD", "111"),
            ]))
            .unwrap();

        JMP_TABLE
            .set(HashMap::from([
                ("null", "000"),
                ("JGT", "001"),
                ("JEQ", "010"),
                ("JGE", "011"),
                ("JLT", "100"),
                ("JNE", "101"),
                ("JLE", "110"),
                ("JMP", "111"),
            ]))
            .unwrap();

        PREDEFINED_SYMBOL_TABLE
            .set(HashMap::from([
                ("SP", 0),
                ("LCL", 1),
                ("ARG", 2),
                ("THIS", 3),
                ("THAT", 4),
                ("R0", 0),
                ("R1", 1),
                ("R2", 2),
                ("R3", 3),
                ("R4", 4),
                ("R5", 5),
                ("R6", 6),
                ("R7", 7),
                ("R8", 8),
                ("R9", 9),
                ("R10", 10),
                ("R11", 11),
                ("R12", 12),
                ("R13", 13),
                ("R14", 14),
                ("R15", 15),
                ("SCREEN", 16384),
                ("KBD", 24576),
            ]))
            .unwrap();

        let mut symbol_table = HashMap::new();

        for symbol in PREDEFINED_SYMBOL_TABLE.get().unwrap().iter() {
            symbol_table.insert(symbol.0.to_owned().to_owned(), symbol.1.to_owned());
        }

        assert_eq!(path.extension().unwrap(), "asm");

        let mut codes = vec![];

        for line in read_to_string(path.clone()).unwrap().lines() {
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

        path.set_extension("hack");

        Self {
            dest_path: path,
            symbol_table,
            codes,
            output: vec![],
            // the next address to be allocated to the variable symbol
            alloc_address: 16,
        }
    }

    pub fn dest_path(&self) -> &PathBuf {
        &self.dest_path
    }

    pub fn run(&mut self) {
        self.process_lable();
        self.parse();
        self.save_binary();
    }

    // First pass through the code to find label symbol like (Xxx)
    fn process_lable(&mut self) {
        let mut no_label_codes = vec![];
        let mut current_line = 0u32;

        for line in self.codes.iter() {
            if line.starts_with('(') {
                let symbol = substr(line, 1, line.len() - 2);
                self.symbol_table.insert(symbol, current_line);
            } else {
                no_label_codes.push(line.to_owned());
                current_line += 1;
            }
        }

        self.codes = no_label_codes;
    }

    // Second pass through the codes to generate binary codes
    fn parse(&mut self) {
        let codes = self.codes.clone();

        for line in codes.iter() {
            if line.starts_with('@') {
                self.parse_a_command(&substr(line, 1, line.len()));
            } else {
                self.parse_c_command(line);
            }
        }
    }

    // Generate binary codes for A command which like @Xxx
    // Note that Xxx can be a symbol or a decimal
    fn parse_a_command(&mut self, command: &str) {
        if command.parse::<u32>().is_ok() {
            writeln!(&mut self.output, "{:016b}", command.parse::<u32>().unwrap()).unwrap();
        } else if self.symbol_table.contains_key(command) {
            let address = self.symbol_table.get(command).unwrap();
            writeln!(&mut self.output, "{:016b}", address).unwrap();
        } else {
            // new variable
            let binary = format!("{:016b}", self.alloc_address);
            self.symbol_table
                .insert(command.to_owned(), self.alloc_address);
            self.alloc_address += 1;
            writeln!(&mut self.output, "{}", binary).unwrap();
        }
    }

    // Generate binary codes for C command which like dest=comp;jmp
    fn parse_c_command(&mut self, command: &str) {
        let mut equal_loc = 0;
        let mut semicolon_loc = command.len();
        let mut dest = "null".to_owned();
        let mut jmp = "null".to_owned();

        if command.find('=').is_some() {
            equal_loc = command.find('=').unwrap();
            dest = substr(command, 0, equal_loc);
        }

        if command.find(';').is_some() {
            semicolon_loc = command.find(';').unwrap();
            jmp = substr(command, semicolon_loc + 1, command.len() - semicolon_loc);
        }

        let comp = if dest == "null" {
            substr(command, 0, semicolon_loc)
        } else {
            substr(command, equal_loc + 1, semicolon_loc - equal_loc + 1)
        };

        writeln!(
            &mut self.output,
            "111{}{}{}",
            COMP_TABLE.get().unwrap().get(comp.as_str()).unwrap(),
            DEST_TABLE.get().unwrap().get(dest.as_str()).unwrap(),
            JMP_TABLE.get().unwrap().get(jmp.as_str()).unwrap(),
        )
        .unwrap();
    }

    fn save_binary(&self) {
        save_file(&self.output, &self.dest_path).unwrap();
    }
}
