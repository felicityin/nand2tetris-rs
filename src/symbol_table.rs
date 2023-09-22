use std::collections::HashMap;
use std::fmt;

use crate::ast::VarType;
use crate::vm_writer::Segment;

pub struct SymbolTable {
    pub table:      HashMap<String, Var>,
    pub kind_index: HashMap<VarKind, u32>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            table:      HashMap::default(),
            kind_index: HashMap::from([
                (VarKind::Static, 0),
                (VarKind::Field, 0),
                (VarKind::Argument, 0),
                (VarKind::Local, 0),
            ]),
        }
    }

    pub fn reset(&mut self) {
        self.table.clear();
        self.kind_index = HashMap::from([
            (VarKind::Static, 0),
            (VarKind::Field, 0),
            (VarKind::Argument, 0),
            (VarKind::Local, 0),
        ]);
    }

    pub fn define(&mut self, name: String, type_: VarType, kind: VarKind) {
        let index = self.kind_index.get(&kind).unwrap().to_owned();
        self.table
            .insert(name.clone(), Var::new(name, type_, kind.clone(), index));
        self.kind_index.insert(kind, index + 1);
    }

    pub fn var_count(&self, kind: VarKind) -> u32 {
        self.kind_index.get(&kind).unwrap().to_owned()
    }

    pub fn get(&self, name: &str) -> Option<&Var> {
        self.table.get(name)
    }
}

#[derive(Clone)]
pub struct Var {
    pub name:  String,
    pub type_: VarType,
    pub kind:  VarKind,
    pub index: u32,
}

impl Var {
    pub fn new(name: String, type_: VarType, scope: VarKind, index: u32) -> Self {
        Self {
            name,
            type_,
            kind: scope,
            index,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum VarKind {
    Static,
    Field,
    Argument,
    Local,
}

impl From<VarKind> for Segment {
    fn from(val: VarKind) -> Self {
        match val {
            VarKind::Static => Segment::Static,
            VarKind::Field => Segment::This,
            VarKind::Argument => Segment::Argument,
            VarKind::Local => Segment::Local,
        }
    }
}

impl fmt::Display for VarKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VarKind::Static => write!(f, "static"),
            VarKind::Field => write!(f, "field"),
            VarKind::Argument => write!(f, "argument"),
            VarKind::Local => write!(f, "local"),
        }
    }
}
