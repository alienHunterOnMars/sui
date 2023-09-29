// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    diagnostics::WarningFilters,
    expansion::ast::{Address, Attributes, Fields, Friend, ModuleIdent, SpecId, Value, Visibility},
    naming::ast::{FunctionSignature, Neighbor, StructDefinition, Type, TypeName_, Type_, Var},
    parser::ast::{BinOp, ConstantName, Field, FunctionName, StructName, UnaryOp, ENTRY_MODIFIER},
    shared::{ast_debug::*, program_info::TypingProgramInfo, unique_map::UniqueMap},
};
use move_ir_types::location::*;
use move_symbol_pool::Symbol;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt,
};

//**************************************************************************************************
// Program
//**************************************************************************************************

#[derive(Debug, Clone)]
pub struct Program {
    pub info: TypingProgramInfo,
    pub inner: Program_,
}

#[derive(Debug, Clone)]
pub struct Program_ {
    pub modules: UniqueMap<ModuleIdent, ModuleDefinition>,
    pub scripts: BTreeMap<Symbol, Script>,
}

//**************************************************************************************************
// Scripts
//**************************************************************************************************

#[derive(Debug, Clone)]
pub struct Script {
    pub warning_filter: WarningFilters,
    // package name metadata from compiler arguments, not used for any language rules
    pub package_name: Option<Symbol>,
    pub attributes: Attributes,
    pub loc: Loc,
    pub immediate_neighbors: UniqueMap<ModuleIdent, Neighbor>,
    pub used_addresses: BTreeSet<Address>,
    pub constants: UniqueMap<ConstantName, Constant>,
    pub function_name: FunctionName,
    pub function: Function,
    // module dependencies referenced in specs
    pub spec_dependencies: BTreeSet<(ModuleIdent, Neighbor)>,
}

//**************************************************************************************************
// Modules
//**************************************************************************************************

#[derive(Debug, Clone)]
pub struct ModuleDefinition {
    pub loc: Loc,
    pub warning_filter: WarningFilters,
    // package name metadata from compiler arguments, not used for any language rules
    pub package_name: Option<Symbol>,
    pub attributes: Attributes,
    pub is_source_module: bool,
    /// `dependency_order` is the topological order/rank in the dependency graph.
    /// `dependency_order` is initialized at `0` and set in the uses pass
    pub dependency_order: usize,
    pub immediate_neighbors: UniqueMap<ModuleIdent, Neighbor>,
    pub used_addresses: BTreeSet<Address>,
    pub friends: UniqueMap<ModuleIdent, Friend>,
    pub structs: UniqueMap<StructName, StructDefinition>,
    pub constants: UniqueMap<ConstantName, Constant>,
    pub functions: UniqueMap<FunctionName, Function>,
    // module dependencies referenced in specs
    pub spec_dependencies: BTreeSet<(ModuleIdent, Neighbor)>,
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

#[derive(PartialEq, Debug, Clone)]
pub enum FunctionBody_ {
    Defined(Sequence),
    Native,
}
pub type FunctionBody = Spanned<FunctionBody_>;

#[derive(PartialEq, Debug, Clone)]
pub struct Function {
    pub warning_filter: WarningFilters,
    // index in the original order as defined in the source file
    pub index: usize,
    pub attributes: Attributes,
    pub visibility: Visibility,
    pub entry: Option<Loc>,
    pub signature: FunctionSignature,
    pub acquires: BTreeMap<StructName, Loc>,
    pub body: FunctionBody,
}

//**************************************************************************************************
// Constants
//**************************************************************************************************

#[derive(PartialEq, Debug, Clone)]
pub struct Constant {
    pub warning_filter: WarningFilters,
    // index in the original order as defined in the source file
    pub index: usize,
    pub attributes: Attributes,
    pub loc: Loc,
    pub signature: Type,
    pub value: Exp,
}

//**************************************************************************************************
// Expressions
//**************************************************************************************************

#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum LValue_ {
    Ignore,
    Var {
        var: Var,
        ty: Box<Type>,
        unused_binding: bool,
    },
    Unpack(ModuleIdent, StructName, Vec<Type>, Fields<(Type, LValue)>),
    BorrowUnpack(
        bool,
        ModuleIdent,
        StructName,
        Vec<Type>,
        Fields<(Type, LValue)>,
    ),
}
pub type LValue = Spanned<LValue_>;
pub type LValueList_ = Vec<LValue>;
pub type LValueList = Spanned<LValueList_>;

#[derive(Debug, PartialEq, Clone)]
pub struct ModuleCall {
    pub module: ModuleIdent,
    pub name: FunctionName,
    pub type_arguments: Vec<Type>,
    pub arguments: Box<Exp>,
    pub parameter_types: Vec<Type>,
    pub acquires: BTreeMap<StructName, Loc>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum BuiltinFunction_ {
    MoveTo(Type),
    MoveFrom(Type),
    BorrowGlobal(bool, Type),
    Exists(Type),
    Freeze(Type),
    Assert(/* is_macro */ bool),
}
pub type BuiltinFunction = Spanned<BuiltinFunction_>;

#[derive(Debug, PartialEq, Clone)]
pub enum UnannotatedExp_ {
    Unit {
        trailing: bool,
    },
    Value(Value),
    Move {
        from_user: bool,
        var: Var,
    },
    Copy {
        from_user: bool,
        var: Var,
    },
    Use(Var),
    Constant(Option<ModuleIdent>, ConstantName),

    ModuleCall(Box<ModuleCall>),
    Builtin(Box<BuiltinFunction>, Box<Exp>),
    Vector(Loc, usize, Box<Type>, Box<Exp>),

    IfElse(Box<Exp>, Box<Exp>, Box<Exp>),
    While(Var, Box<Exp>, Box<Exp>),
    Loop {
        name: Var,
        has_break: bool,
        body: Box<Exp>,
    },
    Block(Sequence),
    Assign(LValueList, Vec<Option<Type>>, Box<Exp>),
    Mutate(Box<Exp>, Box<Exp>),
    Return(Box<Exp>),
    Abort(Box<Exp>),
    Give(Var, Box<Exp>),
    Continue(Var),

    Dereference(Box<Exp>),
    UnaryExp(UnaryOp, Box<Exp>),
    BinopExp(Box<Exp>, BinOp, Box<Type>, Box<Exp>),

    Pack(ModuleIdent, StructName, Vec<Type>, Fields<(Type, Exp)>),
    ExpList(Vec<ExpListItem>),

    Borrow(bool, Box<Exp>, Field),
    TempBorrow(bool, Box<Exp>),
    BorrowLocal(bool, Var),

    Cast(Box<Exp>, Box<Type>),
    Annotate(Box<Exp>, Box<Type>),

    Spec(SpecId, BTreeMap<Var, Type>),

    UnresolvedError,
}
pub type UnannotatedExp = Spanned<UnannotatedExp_>;
#[derive(Debug, PartialEq, Clone)]
pub struct Exp {
    pub ty: Type,
    pub exp: UnannotatedExp,
}
pub fn exp(ty: Type, exp: UnannotatedExp) -> Exp {
    Exp { ty, exp }
}

pub type Sequence = VecDeque<SequenceItem>;
#[derive(Debug, PartialEq, Clone)]
pub enum SequenceItem_ {
    Seq(Box<Exp>),
    Declare(LValueList),
    Bind(LValueList, Vec<Option<Type>>, Box<Exp>),
}
pub type SequenceItem = Spanned<SequenceItem_>;

#[derive(Debug, PartialEq, Clone)]
pub enum ExpListItem {
    Single(Exp, Box<Type>),
    Splat(Loc, Exp, Vec<Type>),
}

pub fn single_item(e: Exp) -> ExpListItem {
    let ty = Box::new(e.ty.clone());
    ExpListItem::Single(e, ty)
}

pub fn splat_item(splat_loc: Loc, e: Exp) -> ExpListItem {
    let ss = match &e.ty {
        sp!(_, Type_::Unit) => vec![],
        sp!(_, Type_::Apply(_, sp!(_, TypeName_::Multiple(_)), ss)) => ss.clone(),
        _ => panic!("ICE splat of non list type"),
    };
    ExpListItem::Splat(splat_loc, e, ss)
}

//**************************************************************************************************
// impls
//**************************************************************************************************

impl BuiltinFunction_ {
    pub fn display_name(&self) -> &'static str {
        use crate::naming::ast::BuiltinFunction_ as NB;
        use BuiltinFunction_ as B;
        match self {
            B::MoveTo(_) => NB::MOVE_TO,
            B::MoveFrom(_) => NB::MOVE_FROM,
            B::BorrowGlobal(false, _) => NB::BORROW_GLOBAL,
            B::BorrowGlobal(true, _) => NB::BORROW_GLOBAL_MUT,
            B::Exists(_) => NB::EXISTS,
            B::Freeze(_) => NB::FREEZE,
            B::Assert(_) => NB::ASSERT_MACRO,
        }
    }
}

//**************************************************************************************************
// Display
//**************************************************************************************************

impl fmt::Display for BuiltinFunction_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

//**************************************************************************************************
// Debug
//**************************************************************************************************

impl AstDebug for Program {
    fn ast_debug(&self, w: &mut AstWriter) {
        self.inner.ast_debug(w)
    }
}

impl AstDebug for Program_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        let Program_ { modules, scripts } = self;

        for (m, mdef) in modules.key_cloned_iter() {
            w.write(&format!("module {}", m));
            w.block(|w| mdef.ast_debug(w));
            w.new_line();
        }

        for (n, s) in scripts {
            w.write(&format!("script {}", n));
            w.block(|w| s.ast_debug(w));
            w.new_line()
        }
    }
}

impl AstDebug for Script {
    fn ast_debug(&self, w: &mut AstWriter) {
        let Script {
            warning_filter,
            package_name,
            attributes,
            loc: _loc,
            immediate_neighbors,
            used_addresses,
            constants,
            function_name,
            function,
            spec_dependencies,
        } = self;
        warning_filter.ast_debug(w);
        if let Some(n) = package_name {
            w.writeln(&format!("{}", n))
        }
        attributes.ast_debug(w);
        for (mident, neighbor) in immediate_neighbors.key_cloned_iter() {
            w.write(&format!("{mident} is"));
            neighbor.ast_debug(w);
            w.writeln(";");
        }
        for addr in used_addresses {
            w.write(&format!("uses address {};", addr));
            w.new_line()
        }
        for (m, neighbor) in spec_dependencies {
            w.write(&format!("spec_dep {m} is"));
            neighbor.ast_debug(w);
            w.writeln(";");
        }
        for cdef in constants.key_cloned_iter() {
            cdef.ast_debug(w);
            w.new_line();
        }
        (*function_name, function).ast_debug(w);
    }
}

impl AstDebug for ModuleDefinition {
    fn ast_debug(&self, w: &mut AstWriter) {
        let ModuleDefinition {
            loc: _,
            warning_filter,
            package_name,
            attributes,
            is_source_module,
            dependency_order,
            immediate_neighbors,
            used_addresses,
            friends,
            structs,
            constants,
            functions,
            spec_dependencies,
        } = self;
        warning_filter.ast_debug(w);
        if let Some(n) = package_name {
            w.writeln(&format!("{}", n))
        }
        attributes.ast_debug(w);
        if *is_source_module {
            w.writeln("library module")
        } else {
            w.writeln("source module")
        }
        w.writeln(&format!("dependency order #{}", dependency_order));
        for (mident, neighbor) in immediate_neighbors.key_cloned_iter() {
            w.write(&format!("{mident} is"));
            neighbor.ast_debug(w);
            w.writeln(";");
        }
        for addr in used_addresses {
            w.write(&format!("uses address {};", addr));
            w.new_line()
        }
        for (m, neighbor) in spec_dependencies {
            w.write(&format!("spec_dep {m} is"));
            neighbor.ast_debug(w);
            w.writeln(";");
        }
        for (mident, _loc) in friends.key_cloned_iter() {
            w.write(&format!("friend {};", mident));
            w.new_line();
        }
        for sdef in structs.key_cloned_iter() {
            sdef.ast_debug(w);
            w.new_line();
        }
        for cdef in constants.key_cloned_iter() {
            cdef.ast_debug(w);
            w.new_line();
        }
        for fdef in functions.key_cloned_iter() {
            fdef.ast_debug(w);
            w.new_line();
        }
    }
}

impl AstDebug for (FunctionName, &Function) {
    fn ast_debug(&self, w: &mut AstWriter) {
        let (
            name,
            Function {
                warning_filter,
                index,
                attributes,
                visibility,
                entry,
                signature,
                acquires,
                body,
            },
        ) = self;
        warning_filter.ast_debug(w);
        attributes.ast_debug(w);
        visibility.ast_debug(w);
        if entry.is_some() {
            w.write(&format!("{} ", ENTRY_MODIFIER));
        }
        if let FunctionBody_::Native = &body.value {
            w.write("native ");
        }
        w.write(&format!("fun#{index} {name}"));
        signature.ast_debug(w);
        if !acquires.is_empty() {
            w.write(" acquires ");
            w.comma(acquires.keys(), |w, s| w.write(&format!("{}", s)));
            w.write(" ");
        }
        match &body.value {
            FunctionBody_::Defined(body) => body.ast_debug(w),
            FunctionBody_::Native => w.writeln(";"),
        }
    }
}

impl AstDebug for (ConstantName, &Constant) {
    fn ast_debug(&self, w: &mut AstWriter) {
        let (
            name,
            Constant {
                warning_filter,
                index,
                attributes,
                loc: _loc,
                signature,
                value,
            },
        ) = self;
        warning_filter.ast_debug(w);
        attributes.ast_debug(w);
        w.write(&format!("const#{index} {name}:"));
        signature.ast_debug(w);
        w.write(" = ");
        value.ast_debug(w);
        w.write(";");
    }
}

impl AstDebug for Sequence {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.block(|w| w.semicolon(self, |w, item| item.ast_debug(w)))
    }
}

impl AstDebug for SequenceItem_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        use SequenceItem_ as I;
        match self {
            I::Seq(e) => e.ast_debug(w),
            I::Declare(sp!(_, bs)) => {
                w.write("let ");
                bs.ast_debug(w);
            }
            I::Bind(sp!(_, bs), expected_types, e) => {
                w.write("let ");
                bs.ast_debug(w);
                w.write(": (");
                expected_types.ast_debug(w);
                w.write(")");
                w.write(" = ");
                e.ast_debug(w);
            }
        }
    }
}

impl AstDebug for UnannotatedExp_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        use UnannotatedExp_ as E;
        match self {
            E::Unit { trailing } if !trailing => w.write("()"),
            E::Unit {
                trailing: _trailing,
            } => w.write("/*()*/"),
            E::Value(v) => v.ast_debug(w),
            E::Move {
                from_user: false,
                var: v,
            } => {
                w.write("move ");
                v.ast_debug(w)
            }
            E::Move {
                from_user: true,
                var: v,
            } => {
                w.write("move@");
                v.ast_debug(w)
            }
            E::Copy {
                from_user: false,
                var: v,
            } => {
                w.write("copy ");
                v.ast_debug(w)
            }
            E::Copy {
                from_user: true,
                var: v,
            } => {
                w.write("copy@");
                v.ast_debug(w)
            }
            E::Use(v) => {
                w.write("use@");
                v.ast_debug(w)
            }
            E::Constant(None, c) => w.write(&format!("{}", c)),
            E::Constant(Some(m), c) => w.write(&format!("{}::{}", m, c)),
            E::ModuleCall(mcall) => {
                mcall.ast_debug(w);
            }
            E::Builtin(bf, rhs) => {
                bf.ast_debug(w);
                w.write("(");
                rhs.ast_debug(w);
                w.write(")");
            }
            E::Vector(_loc, usize, ty, elems) => {
                w.write(format!("vector#{}", usize));
                w.write("<");
                ty.ast_debug(w);
                w.write(">");
                w.write("[");
                elems.ast_debug(w);
                w.write("]");
            }
            E::Pack(m, s, tys, fields) => {
                w.write(&format!("{}::{}", m, s));
                w.write("<");
                tys.ast_debug(w);
                w.write(">");
                w.write("{");
                w.comma(fields, |w, (_, f, idx_bt_e)| {
                    let (idx, (bt, e)) = idx_bt_e;
                    w.write(&format!("({}#{}:", idx, f));
                    bt.ast_debug(w);
                    w.write("): ");
                    e.ast_debug(w);
                });
                w.write("}");
            }
            E::IfElse(b, t, f) => {
                w.write("if (");
                b.ast_debug(w);
                w.write(") ");
                t.ast_debug(w);
                w.write(" else ");
                f.ast_debug(w);
            }
            E::While(name, b, e) => {
                w.write("while @");
                name.ast_debug(w);
                w.write(" (");
                b.ast_debug(w);
                w.write(")");
                e.ast_debug(w);
            }
            E::Loop {
                name,
                has_break,
                body,
            } => {
                w.write("loop");
                if *has_break {
                    w.write("#with_break");
                }
                w.write(" @");
                name.ast_debug(w);
                w.write(" ");
                body.ast_debug(w);
            }
            E::Block(seq) => seq.ast_debug(w),
            E::ExpList(es) => {
                w.write("(");
                w.comma(es, |w, e| e.ast_debug(w));
                w.write(")");
            }

            E::Assign(sp!(_, lvalues), expected_types, rhs) => {
                lvalues.ast_debug(w);
                w.write(": (");
                expected_types.ast_debug(w);
                w.write(") = ");
                rhs.ast_debug(w);
            }

            E::Mutate(lhs, rhs) => {
                w.write("*");
                lhs.ast_debug(w);
                w.write(" = ");
                rhs.ast_debug(w);
            }

            E::Return(e) => {
                w.write("return ");
                e.ast_debug(w);
            }
            E::Abort(e) => {
                w.write("abort ");
                e.ast_debug(w);
            }
            E::Give(name, exp) => {
                w.write("give@");
                name.ast_debug(w);
                w.write(" ");
                exp.ast_debug(w);
            }
            E::Continue(name) => {
                w.write("continue@");
                name.ast_debug(w);
            }
            E::Dereference(e) => {
                w.write("*");
                e.ast_debug(w)
            }
            E::UnaryExp(op, e) => {
                op.ast_debug(w);
                w.write(" ");
                e.ast_debug(w);
            }
            E::BinopExp(l, op, ty, r) => {
                l.ast_debug(w);
                w.write(" ");
                op.ast_debug(w);
                w.write("@");
                ty.ast_debug(w);
                w.write(" ");
                r.ast_debug(w)
            }
            E::Borrow(mut_, e, f) => {
                w.write("&");
                if *mut_ {
                    w.write("mut ");
                }
                e.ast_debug(w);
                w.write(&format!(".{}", f));
            }
            E::TempBorrow(mut_, e) => {
                w.write("&");
                if *mut_ {
                    w.write("mut ");
                }
                e.ast_debug(w);
            }
            E::BorrowLocal(mut_, v) => {
                w.write("&");
                if *mut_ {
                    w.write("mut ");
                }
                v.ast_debug(w);
            }
            E::Cast(e, ty) => {
                w.write("(");
                e.ast_debug(w);
                w.write(" as ");
                ty.ast_debug(w);
                w.write(")");
            }
            E::Annotate(e, ty) => {
                w.write("annot(");
                e.ast_debug(w);
                w.write(": ");
                ty.ast_debug(w);
                w.write(")");
            }
            E::Spec(u, used_locals) => {
                w.write(&format!("spec #{}", u));
                if !used_locals.is_empty() {
                    w.write("uses [");
                    w.comma(used_locals, |w, (n, ty)| w.annotate(|w| n.ast_debug(w), ty));
                    w.write("]");
                }
            }
            E::UnresolvedError => w.write("_|_"),
        }
    }
}

impl AstDebug for Exp {
    fn ast_debug(&self, w: &mut AstWriter) {
        let Exp { ty, exp } = self;
        w.annotate(|w| exp.ast_debug(w), ty)
    }
}

impl AstDebug for ModuleCall {
    fn ast_debug(&self, w: &mut AstWriter) {
        let ModuleCall {
            module,
            name,
            type_arguments,
            parameter_types,
            acquires,
            arguments,
        } = self;
        w.write(&format!("{}::{}", module, name));
        if !acquires.is_empty() || !parameter_types.is_empty() {
            w.write("[");
            if !acquires.is_empty() {
                w.write("acquires: [");
                w.comma(acquires.keys(), |w, s| w.write(&format!("{}", s)));
                w.write("], ");
            }
            if !parameter_types.is_empty() {
                if !acquires.is_empty() {
                    w.write(", ");
                }
                w.write("parameter_types: [");
                parameter_types.ast_debug(w);
                w.write("]");
            }
        }
        w.write("<");
        type_arguments.ast_debug(w);
        w.write(">");
        w.write("(");
        arguments.ast_debug(w);
        w.write(")");
    }
}

impl AstDebug for BuiltinFunction_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        use crate::naming::ast::BuiltinFunction_ as NF;
        use BuiltinFunction_ as F;
        let (n, bt_opt) = match self {
            F::MoveTo(bt) => (NF::MOVE_TO, Some(bt)),
            F::MoveFrom(bt) => (NF::MOVE_FROM, Some(bt)),
            F::BorrowGlobal(true, bt) => (NF::BORROW_GLOBAL_MUT, Some(bt)),
            F::BorrowGlobal(false, bt) => (NF::BORROW_GLOBAL, Some(bt)),
            F::Exists(bt) => (NF::EXISTS, Some(bt)),
            F::Freeze(bt) => (NF::FREEZE, Some(bt)),
            F::Assert(_) => (NF::ASSERT_MACRO, None),
        };
        w.write(n);
        if let Some(bt) = bt_opt {
            w.write("<");
            bt.ast_debug(w);
            w.write(">");
        }
    }
}

impl AstDebug for ExpListItem {
    fn ast_debug(&self, w: &mut AstWriter) {
        match self {
            ExpListItem::Single(e, st) => w.annotate(|w| e.ast_debug(w), st),
            ExpListItem::Splat(_, e, ss) => {
                w.write("~");
                w.annotate(|w| e.ast_debug(w), ss)
            }
        }
    }
}

impl AstDebug for Vec<Option<Type>> {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.comma(self, |w, t_opt| match t_opt {
            Some(t) => t.ast_debug(w),
            None => w.write("%no_exp%"),
        })
    }
}

impl AstDebug for Vec<LValue> {
    fn ast_debug(&self, w: &mut AstWriter) {
        let parens = self.len() != 1;
        if parens {
            w.write("(");
        }
        w.comma(self, |w, a| a.ast_debug(w));
        if parens {
            w.write(")");
        }
    }
}

impl AstDebug for LValue_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        use LValue_ as L;
        match self {
            L::Ignore => w.write("_"),
            L::Var {
                var: v,
                ty: st,
                unused_binding,
            } => w.annotate(
                |w| {
                    v.ast_debug(w);
                    if *unused_binding {
                        w.write("#unused")
                    }
                },
                st,
            ),
            L::Unpack(m, s, tys, fields) => {
                w.write(&format!("{}::{}", m, s));
                w.write("<");
                tys.ast_debug(w);
                w.write(">");
                w.write("{");
                w.comma(fields, |w, (_, f, idx_bt_a)| {
                    let (idx, (bt, a)) = idx_bt_a;
                    w.annotate(|w| w.write(&format!("{}#{}", idx, f)), bt);
                    w.write(": ");
                    a.ast_debug(w);
                });
                w.write("}");
            }
            L::BorrowUnpack(mut_, m, s, tys, fields) => {
                w.write("&");
                if *mut_ {
                    w.write("mut ");
                }
                w.write(&format!("{}::{}", m, s));
                w.write("<");
                tys.ast_debug(w);
                w.write(">");
                w.write("{");
                w.comma(fields, |w, (_, f, idx_bt_a)| {
                    let (idx, (bt, a)) = idx_bt_a;
                    w.annotate(|w| w.write(&format!("{}#{}", idx, f)), bt);
                    w.write(": ");
                    a.ast_debug(w);
                });
                w.write("}");
            }
        }
    }
}
