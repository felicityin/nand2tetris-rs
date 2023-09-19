use std::collections::HashSet;
use std::fmt;
use std::fs::read_to_string;
use std::io::Write;
use std::path::PathBuf;

use crate::utils::{save_file, substr};

use once_cell::sync::OnceCell;

pub static KEYWORDS: OnceCell<HashSet<&str>> = OnceCell::new();
pub static SYMBOLS: OnceCell<HashSet<char>> = OnceCell::new();

pub struct JackTokenizer {
    codes:     Vec<String>,
    dest_path: PathBuf,
    tokens:    Vec<Token>,
}

impl JackTokenizer {
    pub fn new(mut path: PathBuf) -> Self {
        KEYWORDS
            .set(HashSet::from([
                "class",
                "constructor",
                "function",
                "method",
                "field",
                "static",
                "var",
                "int",
                "char",
                "boolean",
                "void",
                "true",
                "false",
                "null",
                "this",
                "let",
                "do",
                "if",
                "else",
                "while",
                "return",
            ]))
            .unwrap();

        SYMBOLS
            .set(HashSet::from([
                '{', '}', '(', ')', '[', ']', '.', ',', ';', '+', '-', '*', '/', '&', '|', '<',
                '>', '=', '~',
            ]))
            .unwrap();

        assert_eq!(path.extension().unwrap(), "jack");

        let mut multi_comments = false;
        let mut codes = vec![];

        for line in read_to_string(path.clone()).unwrap().lines() {
            let line = line.trim();

            if multi_comments {
                if !line.contains("*/") {
                    continue;
                }
                let start = line.find("*/").unwrap() + 2;
                codes.push(substr(line, start, line.len() - start).trim().to_string());
                multi_comments = false;
            } else if line.is_empty() || line.starts_with("//") {
                continue;
            } else if line.contains("//") {
                let end = line.find("//").unwrap();
                codes.push(substr(line, 0, end).trim().to_string());
            } else if line.contains("/*") {
                let start = line.find("/*").unwrap();
                let end = line.find("*/");
                if end.is_none() {
                    multi_comments = true;
                    codes.push(substr(line, 0, start).trim().to_string());
                } else {
                    codes.push(format!(
                        "{}{}",
                        substr(line, 0, start).trim().to_string(),
                        substr(line, end.unwrap() + 2, line.len() - end.unwrap() - 2)
                            .trim()
                            .to_string(),
                    ));
                }
            } else {
                codes.push(line.to_owned());
            }
        }

        path.set_extension("token.xml");

        Self {
            codes,
            dest_path: path,
            tokens: vec![],
        }
    }

    pub fn dest_path(&self) -> &PathBuf {
        &self.dest_path
    }

    pub fn tokens(self) -> Vec<Token> {
        self.tokens
    }

    pub fn tokenize(&mut self) {
        for line in self.codes.iter() {
            let line = line.trim().to_owned();

            if line.is_empty() {
                continue;
            }

            let mut chars = CharStream::new(line.chars().collect());

            while !chars.is_end() {
                let mut c = chars.next();

                if c == ' ' {
                    continue;
                }

                if SYMBOLS.get().unwrap().contains(&c) {
                    self.tokens
                        .push(Token::new(Category::Symbol, c.to_string()));
                } else if c == '\"' {
                    let mut word = String::default();
                    while !chars.is_end() {
                        c = chars.next();
                        if c == '\"' {
                            break;
                        }
                        word.push(c);
                    }
                    self.tokens.push(Token::new(Category::StringConstant, word));
                } else if c.is_numeric() {
                    let mut number = String::from(c);
                    while !chars.is_end() {
                        c = chars.peek();
                        if !c.is_numeric() {
                            break;
                        }
                        number.push(c);
                        chars.next();
                    }
                    self.tokens
                        .push(Token::new(Category::IntegerConstant, number));
                } else {
                    let mut word = String::from(c);
                    while !chars.is_end() {
                        c = chars.peek();
                        if c.is_alphanumeric() {
                            word.push(c);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if KEYWORDS.get().unwrap().contains(word.as_str()) {
                        self.tokens.push(Token::new(Category::Keyword, word));
                    } else {
                        self.tokens.push(Token::new(Category::Identifier, word));
                    }
                }
            }
        }
    }

    pub fn save_file(&self) {
        let mut output: Vec<u8> = vec![];
        writeln!(&mut output, "<tokens>").unwrap();

        for token in self.tokens.iter() {
            writeln!(&mut output, "{}", token.form).unwrap();
        }

        writeln!(&mut output, "</tokens>").unwrap();
        save_file(&output, &self.dest_path).unwrap();
    }
}

struct CharStream {
    s: Vec<char>,
    i: usize,
}

impl CharStream {
    pub fn new(s: Vec<char>) -> Self {
        Self { s, i: 0 }
    }

    pub fn peek(&self) -> char {
        self.s[self.i]
    }

    pub fn next(&mut self) -> char {
        let c = self.s[self.i];
        self.i += 1;
        c
    }

    pub fn is_end(&self) -> bool {
        self.i >= self.s.len()
    }
}

#[derive(Clone)]
pub struct Token {
    pub category:   Category,
    pub value:      String,
    pub is_termial: bool,
    pub form:       String,
}

impl Token {
    pub fn new(category: Category, value: String) -> Self {
        let form = match value.as_str() {
            "<" => format!("<{}> &lt; </{}>", category, category),
            ">" => format!("<{}> &gt; </{}>", category, category),
            "&" => format!("<{}> &amp; </{}>", category, category),
            _ => format!("<{}> {} </{}>", category, value, category),
        };

        Self {
            category,
            value,
            is_termial: true,
            form,
        }
    }

    pub fn unterminal(value: &str, is_begin: bool) -> Self {
        let form = if is_begin {
            format!("<{}>", value)
        } else {
            format!("</{}>", value)
        };

        Self {
            category: Category::Identifier, // unused
            value: value.to_owned(),
            is_termial: false,
            form,
        }
    }
}

#[derive(Clone)]
pub enum Category {
    Symbol,
    StringConstant,
    IntegerConstant,
    Keyword,
    Identifier,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Category::Symbol => write!(f, "symbol"),
            Category::StringConstant => write!(f, "stringConstant"),
            Category::IntegerConstant => write!(f, "integerConstant"),
            Category::Keyword => write!(f, "keyword"),
            Category::Identifier => write!(f, "identifier"),
        }
    }
}
