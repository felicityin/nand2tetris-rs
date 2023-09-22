use std::fmt;
use std::io::Write;
use std::path::PathBuf;

use crate::ast::*;
use crate::symbol_table::{SymbolTable, Var, VarKind};
use crate::utils::save_file;

pub struct VmWriter {
    ast:       Class,
    context:   VmContext,
    vm_writer: VmCommandWriter,
}

impl VmWriter {
    pub fn new(ast: Class) -> Self {
        Self {
            context: VmContext::new(ast.name.clone()),
            ast,
            vm_writer: VmCommandWriter::new(),
        }
    }

    pub fn run(&mut self) {
        self.ast.write_vm(&mut self.context, &mut self.vm_writer);
    }

    pub fn save_file(self, dst_path: PathBuf) {
        assert_eq!(dst_path.extension().unwrap(), "vm");
        save_file(&self.vm_writer.output(), &dst_path).unwrap();
    }
}

pub struct VmContext {
    pub class_name:   String,
    pub class_scope:  SymbolTable,
    pub method_scope: SymbolTable,
    pub method_name:  Option<String>,
    pub method_kind:  Option<SubroutineKind>,
    pub lable_count:  u32,
}

impl VmContext {
    pub fn new(class_name: String) -> Self {
        Self {
            class_name,
            class_scope: SymbolTable::new(),
            method_scope: SymbolTable::new(),
            method_name: None,
            method_kind: None,
            lable_count: 0,
        }
    }

    pub fn start_subroutine(&mut self, name: String, kind: SubroutineKind) {
        self.method_scope.reset();
        self.method_name = Some(name);
        self.method_kind = Some(kind);
    }

    pub fn define_class_var(&mut self, name: String, type_: VarType, var_kind: VarKind) {
        self.class_scope.define(name, type_, var_kind);
    }

    pub fn define_method_var(&mut self, name: String, type_: VarType, var_kind: VarKind) {
        self.method_scope.define(name, type_, var_kind);
    }

    pub fn inc_label(&mut self) -> u32 {
        let label_count = self.lable_count;
        self.lable_count += 1;
        label_count
    }

    pub fn current_function(&self) -> String {
        let subroutine_name = self
            .method_name
            .as_ref()
            .expect("subroutine name is not defined");
        format!("{}.{}", self.class_name, subroutine_name)
    }

    pub fn function_kind(&self) -> SubroutineKind {
        self.method_kind
            .as_ref()
            .expect("subroutine kind is not defined")
            .to_owned()
    }

    pub fn find(&self, name: &str) -> bool {
        self.method_scope.get(name).is_some() || self.class_scope.get(name).is_some()
    }

    pub fn get(&self, name: &str) -> Option<&Var> {
        let var = self.method_scope.get(name);
        if var.is_some() {
            var
        } else {
            self.class_scope.get(name)
        }
    }

    pub fn object_fields_count(&self) -> u32 {
        self.class_scope.var_count(VarKind::Field)
    }

    pub fn local_var_count(&self) -> u32 {
        self.method_scope.var_count(VarKind::Local)
    }

    pub fn param_count(&self) -> u32 {
        self.method_scope.var_count(VarKind::Argument)
    }

    pub fn local_variables(&self) -> Vec<&Var> {
        self.method_scope.table.values().collect()
    }
}

pub trait VmWrite {
    fn write_vm(&self, context: &mut VmContext, vm_output: &mut VmCommandWriter);
}

impl VmWrite for Class {
    fn write_vm(&self, context: &mut VmContext, vm_output: &mut VmCommandWriter) {
        for var in self.vars.iter() {
            var.names.iter().for_each(|id| {
                context.define_class_var(id.to_owned(), var.type_.clone(), var.kind.clone().into());
            })
        }

        for subroutine in self.subroutines.iter() {
            subroutine.write_vm(context, vm_output);
        }
    }
}

impl VmWrite for SubroutineDec {
    fn write_vm(&self, context: &mut VmContext, vm_output: &mut VmCommandWriter) {
        context.start_subroutine(self.name.clone(), self.kind.clone());

        // Pass a reference to the manipulated object as a hidden argument of the called
        // method Compile b.mult(5) as if it were written as mult(b, 5)
        if self.kind == SubroutineKind::Method {
            context.define_method_var(
                "this".to_string(),
                VarType::Class(context.class_name.clone()),
                VarKind::Argument,
            );
        }

        for param in self.params.iter() {
            context.define_method_var(
                param.name.clone(),
                param.var_type.clone(),
                VarKind::Argument,
            );
        }

        self.body.write_vm(context, vm_output);
    }
}

impl VmWrite for SubroutineBody {
    fn write_vm(&self, context: &mut VmContext, vm_output: &mut VmCommandWriter) {
        for var in self.local_vars.iter() {
            var.names.iter().cloned().for_each(|id| {
                context.define_method_var(id, var.type_.clone(), VarKind::Local);
            });
        }

        vm_output.write_function(&context.current_function(), context.param_count());

        for _ in 0..context.local_var_count() {
            vm_output.write_push(Segment::Constant, 0);
        }

        let kind = context.function_kind();
        // this = manipulated object
        if kind == SubroutineKind::Method {
            vm_output.write_push(Segment::Argument, 0);
            vm_output.write_pop(Segment::Pointer, 0);
        }
        // this = alloc(fields count)
        if kind == SubroutineKind::Constructor {
            vm_output.write_push(Segment::Constant, context.object_fields_count());
            vm_output.write_call("Memory.alloc", 1);
            vm_output.write_pop(Segment::Pointer, 0);
        }

        for var in context.local_variables() {
            if var.kind == VarKind::Local && var.type_ == VarType::Class("String".to_string()) {
                vm_output.write_push(Segment::Constant, var.name.len() as u32);
                vm_output.write_call("String.new", 1);
                for c in var.name.chars() {
                    vm_output.write_push(Segment::Constant, c as u32);
                    vm_output.write_call("String.appendChar", 2);
                }
                vm_output.write_pop(Segment::Local, var.index);
            }
        }

        for statement in self.body.iter() {
            statement.write_vm(context, vm_output);
        }
    }
}

impl VmWrite for Statement {
    fn write_vm(&self, context: &mut VmContext, vm_output: &mut VmCommandWriter) {
        match self {
            Statement::Let(v) => v.write_vm(context, vm_output),
            Statement::If(v) => v.write_vm(context, vm_output),
            Statement::While(v) => v.write_vm(context, vm_output),
            Statement::Do(v) => v.write_vm(context, vm_output),
            Statement::Return(v) => v.write_vm(context, vm_output),
        }
    }
}

impl VmWrite for LetStatement {
    fn write_vm(&self, context: &mut VmContext, vm_output: &mut VmCommandWriter) {
        self.right_expr.write_vm(context, vm_output);

        if let Some(ref index) = self.array_index {
            index.write_vm(context, vm_output);

            // Get var's base address
            let var = context.get(&self.var_name).unwrap();
            vm_output.write_push(var.kind.to_owned().into(), var.index);

            Op::Add.write_vm(context, vm_output);

            // Set that's base to (var + index)
            vm_output.write_pop(Segment::Pointer, 1);

            // *(var + index) = right expr
            vm_output.write_pop(Segment::That, 0);
        } else {
            // var = right expr
            let var = context.get(&self.var_name).unwrap();
            vm_output.write_pop(var.kind.to_owned().into(), var.index);
        }
    }
}

impl VmWrite for IfStatement {
    fn write_vm(&self, context: &mut VmContext, vm_output: &mut VmCommandWriter) {
        let then_label = format!("if_{}", context.inc_label());
        let end_label = format!("fi_{}", context.inc_label());

        self.cond.write_vm(context, vm_output);
        vm_output.write_if_goto(&then_label);

        if let Some(ref else_body) = self.else_body {
            for statement in else_body.iter() {
                statement.write_vm(context, vm_output);
            }
        }
        vm_output.write_goto(&end_label);

        vm_output.write_label(&then_label);
        for statement in self.if_body.iter() {
            statement.write_vm(context, vm_output);
        }

        vm_output.write_label(&end_label);
    }
}

impl VmWrite for DoStatement {
    fn write_vm(&self, context: &mut VmContext, vm_output: &mut VmCommandWriter) {
        self.subroutine_call.write_vm(context, vm_output);
        vm_output.write_pop(Segment::Temp, 0);
    }
}

impl VmWrite for ReturnStatement {
    fn write_vm(&self, context: &mut VmContext, vm_output: &mut VmCommandWriter) {
        for var in context.local_variables() {
            if var.kind == VarKind::Local && var.type_ == VarType::Class("String".to_string()) {
                vm_output.write_push(Segment::Local, var.index);
                vm_output.write_call("String.dispose", 1);
            }
        }

        if let Some(ref expr) = self.expr {
            expr.write_vm(context, vm_output);
        } else {
            vm_output.write_push(Segment::Constant, 0);
        }

        vm_output.write_return();
    }
}

impl VmWrite for WhileStatement {
    fn write_vm(&self, context: &mut VmContext, vm_output: &mut VmCommandWriter) {
        let loop_start_label = format!("loop_start_{}", context.inc_label());
        let loop_end_label = format!("loop_end_{}", context.inc_label());

        vm_output.write_label(&loop_start_label);

        self.cond.write_vm(context, vm_output);
        vm_output.write_if_goto(&loop_end_label);

        for statement in self.body.iter() {
            statement.write_vm(context, vm_output);
        }
        vm_output.write_goto(&loop_start_label);

        vm_output.write_label(&loop_end_label);
    }
}

impl VmWrite for SubroutineCall {
    fn write_vm(&self, context: &mut VmContext, vm_output: &mut VmCommandWriter) {
        match self {
            Self::Internal(v) => v.write_vm(context, vm_output),
            Self::External(v) => v.write_vm(context, vm_output),
        }
    }
}

impl VmWrite for InternalCall {
    fn write_vm(&self, context: &mut VmContext, vm_output: &mut VmCommandWriter) {
        // The first argument
        vm_output.write_push(Segment::Pointer, 0);

        // Others arguments
        self.args.write_vm(context, vm_output);

        let func_name = format!("{}.{}", context.class_name, self.name);
        vm_output.write_call(&func_name, self.args.0.len() as u32);
    }
}

impl VmWrite for ExternalCall {
    fn write_vm(&self, context: &mut VmContext, vm_output: &mut VmCommandWriter) {
        match context.get(&self.name).cloned() {
            Some(var) => {
                match var.type_ {
                    VarType::Class(ref class) => {
                        // Arguments
                        vm_output.write_push(var.kind.clone().into(), var.index);
                        self.args.write_vm(context, vm_output);

                        let func_name = format!("{}.{}", class, self.name);
                        vm_output.write_call(&func_name, self.args.0.len() as u32);
                    }
                    _ => panic!("invald var: {}, type: {:?}", var.name, var.type_),
                }
            }
            None => {
                self.args.write_vm(context, vm_output);

                let func_name = format!("{}.{}", self.name, self.subroutine_name);
                vm_output.write_call(&func_name, self.args.0.len() as u32);
            }
        }
    }
}

impl VmWrite for Args {
    fn write_vm(&self, context: &mut VmContext, vm_output: &mut VmCommandWriter) {
        for arg in self.0.iter() {
            arg.write_vm(context, vm_output);
        }
    }
}

impl VmWrite for Expression {
    fn write_vm(&self, context: &mut VmContext, vm_output: &mut VmCommandWriter) {
        self.term.write_vm(context, vm_output);

        for op_term in self.op_terms.iter() {
            op_term.term.write_vm(context, vm_output);
            op_term.op.write_vm(context, vm_output);
        }
    }
}

impl VmWrite for Term {
    fn write_vm(&self, context: &mut VmContext, vm_output: &mut VmCommandWriter) {
        match self {
            Term::IntegerConst(v) => vm_output.write_push(Segment::Constant, *v),
            Term::KeywordConst(v) => match v {
                KeywordConstant::Null | KeywordConstant::False => {
                    vm_output.write_push(Segment::Constant, 0)
                }
                KeywordConstant::True => {
                    vm_output.write_push(Segment::Constant, 0);
                    UnaryOp::Not.write_vm(context, vm_output);
                }
                KeywordConstant::This => vm_output.write_push(Segment::Pointer, 0),
            },
            Term::StringConst(v) => {
                if !context.find(v) {
                    context.define_method_var(
                        v.to_string(),
                        VarType::Class("String".to_string()),
                        VarKind::Local,
                    );
                }
                let var = context.get(v).unwrap();
                vm_output.write_push(Segment::Local, var.index.to_owned());
            }
            Term::VarName(v) => {
                let var = context.get(v).unwrap();
                vm_output.write_push(var.kind.clone().into(), var.index);
            }
            Term::Expression(v) => v.write_vm(context, vm_output),
            Term::Array(v) => v.write_vm(context, vm_output),
            Term::UnaryExpression(v) => v.write_vm(context, vm_output),
            Term::SubRoutineCall(v) => v.write_vm(context, vm_output),
        }
    }
}

impl VmWrite for UnaryExpression {
    fn write_vm(&self, context: &mut VmContext, vm_output: &mut VmCommandWriter) {
        self.term.write_vm(context, vm_output);
        self.unary_op.write_vm(context, vm_output);
    }
}

impl VmWrite for Array {
    fn write_vm(&self, context: &mut VmContext, vm_output: &mut VmCommandWriter) {
        self.index.write_vm(context, vm_output);

        // Get var's base address
        let var = context.get(&self.name).unwrap();
        vm_output.write_push(var.kind.to_owned().into(), var.index);

        Op::Add.write_vm(context, vm_output);

        // Set that's base to (var + index)
        vm_output.write_pop(Segment::Pointer, 1);

        // *(var + index) = right expr
        vm_output.write_push(Segment::That, 0);
    }
}

impl VmWrite for Op {
    fn write_vm(&self, _context: &mut VmContext, vm_output: &mut VmCommandWriter) {
        match self {
            Op::Add => vm_output.write_arithmetic("add"),
            Op::Minus => vm_output.write_arithmetic("sub"),
            Op::Multiply => vm_output.write_call("Math.multiply", 2),
            Op::Divid => vm_output.write_call("Math.divide", 2),
            Op::And => vm_output.write_arithmetic("and"),
            Op::Or => vm_output.write_arithmetic("or"),
            Op::Greater => vm_output.write_arithmetic("gt"),
            Op::Less => vm_output.write_arithmetic("lt"),
            Op::Euqal => vm_output.write_arithmetic("eq"),
        }
    }
}

impl VmWrite for UnaryOp {
    fn write_vm(&self, _context: &mut VmContext, vm_output: &mut VmCommandWriter) {
        match self {
            UnaryOp::Neg => vm_output.write_arithmetic("neg"),
            UnaryOp::Not => vm_output.write_arithmetic("not"),
        }
    }
}

pub struct VmCommandWriter {
    output: Vec<u8>,
}

impl VmCommandWriter {
    pub fn new() -> Self {
        Self { output: vec![] }
    }

    pub fn write_push(&mut self, segment: Segment, index: u32) {
        writeln!(&mut self.output, "push {} {}", segment, index).unwrap();
    }

    pub fn write_pop(&mut self, segment: Segment, index: u32) {
        writeln!(&mut self.output, "push {} {}", segment, index).unwrap();
    }

    pub fn write_arithmetic(&mut self, command: &str) {
        writeln!(&mut self.output, "{}", command).unwrap();
    }

    pub fn write_label(&mut self, label: &str) {
        writeln!(&mut self.output, "label {}", label).unwrap();
    }

    pub fn write_goto(&mut self, label: &str) {
        writeln!(&mut self.output, "goto {}", label).unwrap();
    }

    pub fn write_if_goto(&mut self, label: &str) {
        writeln!(&mut self.output, "if-goto {}", label).unwrap();
    }

    pub fn write_call(&mut self, name: &str, args_count: u32) {
        writeln!(&mut self.output, "call {} {}", name, args_count).unwrap();
    }

    pub fn write_function(&mut self, name: &str, args_count: u32) {
        writeln!(&mut self.output, "function {} {}", name, args_count).unwrap();
    }

    pub fn write_return(&mut self) {
        writeln!(&mut self.output, "return").unwrap();
    }

    pub fn output(self) -> Vec<u8> {
        self.output
    }
}

pub enum Segment {
    Argument,
    Local,
    Static,
    Constant,
    This,
    That,
    Pointer,
    Temp,
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Segment::Argument => write!(f, "argument"),
            Segment::Local => write!(f, "local"),
            Segment::Static => write!(f, "static"),
            Segment::Constant => write!(f, "constant"),
            Segment::This => write!(f, "this"),
            Segment::That => write!(f, "that"),
            Segment::Pointer => write!(f, "pointer"),
            Segment::Temp => write!(f, "temp"),
        }
    }
}
