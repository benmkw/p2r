#![forbid(unsafe_code)]
#![feature(iter_intersperse)]
#![feature(iter_map_windows)]
#![feature(let_chains)]

use rustpython_parser::ast::{
    ArgWithDefault, BoolOp, CmpOp, Constant, ExceptHandlerExceptHandler, Expr, ExprAttribute,
    ExprBinOp, ExprBoolOp, ExprCall, ExprCompare, ExprConstant, ExprContext, ExprDict,
    ExprDictComp, ExprFormattedValue, ExprGeneratorExp, ExprIfExp, ExprJoinedStr, ExprLambda,
    ExprList, ExprListComp, ExprName, ExprNamedExpr, ExprSet, ExprSetComp, ExprSlice, ExprStarred,
    ExprSubscript, ExprTuple, ExprUnaryOp, MatchCase, Mod, Operator, Pattern, PatternMatchValue,
    Stmt, StmtAnnAssign, StmtAssert, StmtAssign, StmtAugAssign, StmtClassDef, StmtDelete, StmtExpr,
    StmtFor, StmtFunctionDef, StmtIf, StmtImport, StmtImportFrom, StmtMatch, StmtRaise, StmtReturn,
    StmtTry, StmtTypeAlias, StmtWhile, UnaryOp,
};
use std::{fmt::Write, ops::Deref};

mod util;
use util::PaddedT;

type TResult<T> = Result<T, TranspileError>;

#[derive(Debug, Clone)]
pub struct TranspileError {
    pub file: &'static str,
    pub line: u32,
}

impl std::fmt::Display for TranspileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Error at {file}:{line}",
            file = self.file,
            line = self.line
        ))
    }
}

impl std::error::Error for TranspileError {}

impl TranspileError {
    pub fn to_link(&self) -> String {
        format!(
            "please, go to: https:://github.com/benmkw/p2r/blob/main/{file}#L{line} and help improve this crate",
            file = self.file,
            line = self.line
        )
    }
}

#[macro_export]
macro_rules! todo_link {
    () => {
        TranspileError {
            file: file!(),
            line: line!(),
        }
    };
}

#[derive(Debug)]
pub enum ParseError {
    TranspileError(TranspileError),
    ParseError(rustpython_parser::ParseError),
}

impl From<TranspileError> for ParseError {
    fn from(value: TranspileError) -> Self {
        Self::TranspileError(value)
    }
}

pub fn fmt(code: &str) -> String {
    if let Ok(t) = syn::parse_file(code) {
        prettyplease::unparse(&t)
    } else {
        code.to_string()
    }
}

#[cfg(test)]
mod test;

pub fn p2r(ast: &str, ctx: &mut Ctx) -> Result<String, ParseError> {
    let mut total = String::new();

    let ast = rustpython_parser::parse(ast, rustpython_parser::Mode::Interactive, "./")
        .map_err(ParseError::ParseError)?;
    let body = match ast {
        Mod::Module(_) => Err(todo_link!()),
        Mod::Interactive(body) => Ok(body),
        Mod::Expression(_) => Err(todo_link!()),
        Mod::FunctionType(_) => Err(todo_link!()),
    }?;

    for b in body.body {
        let rust = r_s(&b, ctx)?;
        total += &rust;
        if b.is_expr_stmt() {
            // TOOD this handling of expr is a bit hacky?
            total += ";\n"
        }
    }

    // TODO move the imports and prelude to the front?
    total += &format!(
        "{}\n{}",
        ctx.imports.gen_imports(),
        ctx.imports.gen_prelude(),
    );

    Ok(total)
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum PyMath {
    Sin,
    Cos,
    Pow,
    Pi,
    Abs,
    Sqrt,
}

impl<S: AsRef<str>> From<S> for PyMath {
    fn from(value: S) -> Self {
        let str = value.as_ref();
        match str {
            "sin" => Self::Sin,
            "cos" => Self::Cos,
            "pow" => Self::Pow,
            "pi" => Self::Pi,
            "abs" => Self::Abs,
            "sqrt" => Self::Sqrt,
            _ => panic!("could not find {str}"),
        }
    }
}

impl PyMath {
    /// polyfill for the python math module
    ///
    /// this can all be inlined into the callers using rust-analyzer
    fn to_rust(&self) -> &'static str {
        match self {
            PyMath::Sin => indoc::indoc! {"
                #[inline(always)] pub fn sin(v : f64) -> f64 { v.sin() }
            "},
            PyMath::Cos => indoc::indoc! {"
                #[inline(always)] pub fn cos(v : f64) -> f64 { v.cos() }
            "},
            PyMath::Pow => indoc::indoc! {"
                #[inline(always)] pub fn pow(a : f64, b : f64) -> f64 { a.powf(b) }
            "},
            PyMath::Abs => indoc::indoc! {"
                #[inline(always)] pub fn abs(a : f64) -> f64 { a.abs() }
            "},
            PyMath::Sqrt => indoc::indoc! {"
                #[inline(always)] pub fn sqrt(a : f64) -> f64 { a.sqrt() }
            "},
            PyMath::Pi => indoc::indoc! {"pub const pi: f64 = std::f64::consts::PI;"},
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum Promotion {
    #[default]
    None,
    /// functions which return Optional[T] can use .into() in rust
    Into,
    /// returning numpy arrays need py arg and need into_pyarray on return
    IntoPyArray,
}

/// Mapping of:
/// name of the imported function -> mapping to its alias (if given)
#[derive(Debug, Clone, Default)]
pub struct Imports {
    // used BTreeMap for deterministic iteration order
    // could just a hashmap with determinsic hasher as well
    pub math: std::collections::BTreeMap<String, Option<String>>,
    // pub called_math_fns: Vec<PyMath>, // or Vec<String>
    /// fns called as math.foo without `from` import
    pub math_import_name: Option<String>,

    pub functools: std::collections::BTreeMap<String, Option<String>>,
    pub itertools: std::collections::BTreeMap<String, Option<String>>,
}

impl Imports {
    fn gen_prelude(&self) -> String {
        if self.math.is_empty() {
            return String::new();
        }

        let mut res = "\nmod prelude {\n".to_string();
        for (name, _rename) in &self.math {
            let rust = PyMath::from(name).to_rust();
            res += rust;
            res += "\n"
        }

        res += "\n}\n";
        res
    }

    fn gen_imports(&self) -> String {
        if self.math.is_empty() {
            return String::new();
        }

        let imports = &self
            .math
            .iter()
            .map(|(name, rename)| {
                if let Some(rename) = rename {
                    format!("{name} as {rename}")
                } else {
                    name.to_string()
                }
            })
            .intersperse_with(|| ",".to_string())
            .collect::<String>();
        format!("use prelude::{{{imports}}};")
    }
}

#[derive(Debug, Clone, Default)]
pub struct Ctx {
    /// List of mappings of name -> member names
    pub classes: Vec<(String, Vec<String>)>,
    pub enums: Vec<String>,
    pub in_enum: bool,
    pub declare_var_mut: bool,
    pub ret_needs_promotion: Promotion,
    /// List of arguments (name, type_comment) which are np arrays
    pub numpy_array_args: Vec<(String, String)>,
    pub imports: Imports,
}

impl Ctx {
    /// None -> class not found
    ///
    /// empty Slice -> no members
    fn get_class_members(&self, class: &str) -> Option<&[String]> {
        self.classes
            .iter()
            .find(|s| s.0.as_str() == class)
            .map(|v| v.1.as_slice())
    }

    fn has_enum(&self, e: &str) -> bool {
        self.enums.iter().any(|s| s.as_str() == e)
    }
}

/// convert statement
fn r_s(node: &Stmt, ctx: &mut Ctx) -> TResult<String> {
    match node {
        Stmt::FunctionDef(StmtFunctionDef {
            name,
            args,
            body,
            decorator_list: _,
            returns,
            type_comment: _,
            range: _,
            type_params: _,
        }) => {
            let args = &args
                .args
                .iter()
                .map(|a| r_a(a, ctx))
                .intersperse_with(|| Ok(", ".to_string()))
                .collect::<TResult<String>>()?;

            let (args, lifetimes) = if ctx.numpy_array_args.is_empty() {
                (args.to_string(), String::new())
            } else {
                (
                    format!("py: pyo3::Python<'py>, {args}"),
                    "<'py>".to_string(),
                )
            };

            let ret_type = match returns {
                Some(r) => r_annotation(r)?,
                None => "()".to_string(),
            };

            // TODO do this with types instead of string comparisons?
            if ret_type.starts_with("Option<") {
                ctx.ret_needs_promotion = Promotion::Into;
            }
            let ret_type = if ret_type.starts_with("numpy::") {
                ctx.ret_needs_promotion = Promotion::IntoPyArray;
                format!("&'py {ret_type}")
            } else {
                ret_type
            };

            let pyo3_conversions =
                ctx.numpy_array_args
                    .iter()
                    .fold(String::new(), |mut output, (name, _t)| {
                        let _ = write!(output, "let {name} = {name}.as_array().to_owned();");
                        output
                    });

            let mut doc_comment = None;
            let body = body
                .iter()
                .map(|s| {
                    if let Some(Some(Some(doc))) = s
                        .as_expr_stmt()
                        .map(|e| e.value.as_constant_expr().map(|c| c.value.as_str()))
                    {
                        // free standing string
                        // -> doc comment
                        doc_comment = Some(format!("/*! {doc} */"));
                        Ok(String::new())
                    } else {
                        r_s(s, ctx)
                    }
                })
                .collect::<TResult<String>>()?;

            Ok(format!(
                "{doc}\nfn {n}{lifetimes}({args}) -> {ret_type} {{\n{pyo3_conversions}\n\n{body}}}\n",
                n = name,
                doc = doc_comment.unwrap_or_default()
            ))
        }
        Stmt::ClassDef(StmtClassDef {
            name,
            body,
            range: _,
            bases,
            keywords: _,
            decorator_list: _,
            type_params: _,
        }) => {
            let mut defs = vec![];

            let struct_enum_def = if bases
                .iter()
                .filter_map(|v| v.as_name_expr())
                .any(|v| v.id.as_str() == "Enum")
            {
                // enum
                ctx.enums.push(name.to_string());

                let mut fields = vec![];

                for b in body.iter() {
                    if let Some(field) = b.clone().assign_stmt() {
                        fields.push(field);
                    } else if let Some(def) = b.clone().function_def_stmt() {
                        defs.push(def)
                    } else {
                        dbg!(b);
                        return Err(todo_link!());
                    }
                }

                ctx.in_enum = true;
                let res = format!(
                    "\n#[derive(Debug, Clone)]\nenum {name} {{\n{body}\n}}",
                    body = fields
                        .iter()
                        // TODO this reboxing is not very elegant
                        .map(|v| r_s(&Stmt::Assign(v.clone()), ctx))
                        .intersperse_with(|| Ok(",\n".to_string()))
                        .collect::<TResult<String>>()?
                );
                ctx.in_enum = false;
                res
            } else {
                // struct
                let mut typed_fields = vec![];
                let mut field_names = vec![];

                for b in body.iter() {
                    if let Some(field) = b.clone().ann_assign_stmt() {
                        field_names.push(r_e(&field.target, ctx)?);
                        // TODO r_annotation?
                        typed_fields.push(r_s(&Stmt::AnnAssign(field.clone()), ctx)?);
                    } else if let Some(def) = b.clone().function_def_stmt() {
                        defs.push(def)
                    } else {
                        return Err(todo_link!());
                    }
                }

                ctx.classes.push((name.to_string(), field_names));

                format!(
                    "\n#[derive(Debug, Clone)]\nstruct {name} {{\n{body}\n}}",
                    body = typed_fields
                        .iter()
                        .cloned()
                        .intersperse(",\n".to_string())
                        .collect::<String>()
                )
            };

            let impls = if defs.is_empty() {
                String::new()
            } else {
                // TODO impl new
                format!(
                    "impl {name} {{\n{impls} }}\n",
                    impls = defs
                        .iter()
                        .map(|v| r_s(&Stmt::FunctionDef(v.clone()), ctx))
                        .intersperse_with(|| Ok("\n".to_string()))
                        .collect::<TResult<String>>()?
                )
            };

            Ok(format!("{struct_enum_def}\n{impls}"))
        }
        Stmt::Return(StmtReturn { value, range: _ }) => {
            if let Some(v) = value {
                let v = r_e(v, ctx)?;
                match ctx.ret_needs_promotion {
                    Promotion::None => Ok(format!("return {v};\n")),
                    Promotion::Into => Ok(format!("return ({v}).into();\n")),
                    Promotion::IntoPyArray => Ok(format!("return ({v}).into_pyarray(py);\n")),
                }
            } else {
                Err(todo_link!())
            }
        }
        Stmt::Delete(StmtDelete { range: _, targets }) => {
            // TODO for copy types we could add a call like `_ = name; to remove them from the scope`
            Ok(targets
                .iter()
                .map(|t| r_e(t, ctx))
                .collect::<TResult<Vec<String>>>()?
                .iter()
                .cloned()
                .map(|name| format!("drop({name})"))
                .intersperse_with(|| ";".to_string())
                .collect::<String>())
        }
        Stmt::Assign(StmtAssign {
            targets,
            value,
            type_comment: _,
            range: _,
        }) => {
            if targets.len() != 1 {
                return Err(todo_link!());
            }

            if ctx.in_enum {
                Ok(format!(
                    "{t} = {v}",
                    t = r_e(&targets[0], ctx)?,
                    v = r_e(value, ctx)?
                ))
            } else {
                let v = r_e(value, ctx)?;
                let t = r_e(&targets[0], ctx)?;
                if t.starts_with("self") {
                    Ok(format!("{t} = {v};\n"))
                } else {
                    ctx.declare_var_mut = true;
                    let t = r_e(&targets[0], ctx)?;
                    ctx.declare_var_mut = false;
                    Ok(format!("let {t} = {v};\n"))
                }
            }
        }
        Stmt::AugAssign(StmtAugAssign {
            range: _,
            target,
            op,
            value,
        }) => Ok(format!(
            "{l} {o}= {r};",
            l = r_e(target, ctx)?,
            o = r_o(op)?,
            r = r_e(value, ctx)?
        )),
        Stmt::AnnAssign(StmtAnnAssign {
            range: _,
            target,
            annotation,
            value: _,
            simple: _,
        }) => {
            // this could be used to impl Default for this struct
            // dbg!(&value);
            Ok(format!(
                "{target} : {annotation}",
                target = r_e(target, ctx)?,
                annotation = r_annotation(annotation)?
            ))
        }
        Stmt::For(StmtFor {
            target,
            iter,
            body,
            orelse,
            type_comment: _,
            range: _,
        }) => {
            // https://book.pythontips.com/en/latest/for_-_else.html#else-clause
            if !orelse.is_empty() {
                return Err(todo_link!());
            }

            let iter = r_e(iter, ctx)?;
            // TODO translate `target` into nested tuple if the iter is a zip
            Ok(format!(
                "for {target} in {iter} {{\n{body}}}\n",
                target = r_e(target, ctx)?,
                body = body
                    .iter()
                    .map(|s| r_s(s, ctx))
                    .collect::<TResult<Vec<String>>>()?
                    .iter()
                    .cloned()
                    .padded(";\n".to_string())
                    .collect::<String>(),
            ))
        }
        Stmt::While(StmtWhile {
            range: _,
            test,
            body,
            orelse,
        }) => {
            let orelse = orelse
                .iter()
                .map(|s| r_s(s, ctx))
                .collect::<TResult<Vec<String>>>()?
                .iter()
                .cloned()
                .padded(";\n".to_string())
                .collect::<String>();

            let test = r_e(test, ctx)?;
            let orelse = if orelse.is_empty() {
                String::new()
            } else {
                format!("if !({test}) {{{orelse}}}")
            };

            Ok(format!(
                "while {test} {{\n{body}\n}}\n{orelse}",
                body = body
                    .iter()
                    .map(|s| r_s(s, ctx))
                    .intersperse_with(|| Ok("\n".to_string()))
                    .collect::<TResult<String>>()?
            ))
        }
        Stmt::If(StmtIf {
            test,
            body,
            orelse,
            range: _,
        }) => {
            // https://docs.python.org/3/library/ast.html#ast.If
            let test = r_e(test, ctx)?;
            // todo
            let body = body
                .iter()
                .map(|s| r_s(s, ctx))
                .collect::<TResult<Vec<String>>>()?
                .iter()
                .cloned()
                .padded(";\n".to_string())
                .collect::<String>();

            let orelse = orelse
                .iter()
                .map(|s| r_s(s, ctx))
                .collect::<TResult<Vec<String>>>()?
                .iter()
                .cloned()
                .padded(";\n".to_string())
                .collect::<String>();

            Ok(if orelse.is_empty() {
                format!("if {test} {{\n{body}\n}}\n")
            } else {
                format!("if {test} {{\n{body}\n}} else {{{orelse}}}\n")
            })
        }
        Stmt::Match(StmtMatch {
            range: _,
            subject,
            cases,
        }) => Ok(format!(
            "match {subject} {{\n{cases}\n}}",
            subject = r_e(subject, ctx)?,
            cases = cases
                .iter()
                .map(
                    |MatchCase {
                         range: _,
                         pattern,
                         guard,
                         body,
                     }| {
                        if guard.is_some() {
                            // "https://peps.python.org/pep-0622/#guards"
                            Err(todo_link!())
                        } else {
                            Ok(format!(
                                "{pattern} => {{ {body} }},",
                                pattern = r_p(pattern, ctx)?,
                                body = body
                                    .iter()
                                    .map(|s| r_s(s, ctx))
                                    .collect::<TResult<String>>()?
                            ))
                        }
                    }
                )
                .collect::<TResult<String>>()?
        )),
        Stmt::Raise(StmtRaise {
            range: _,
            exc,
            cause,
        }) => {
            if cause.is_some() {
                // https://docs.python.org/3/library/ast.html#ast.Raise
                // raise x from y
                return Err(todo_link!());
            }

            if let Some(exc) = exc {
                // TODO create an error enum member with this
                // name automatically
                // this is a bit hard, because we'd ideally collect all
                // possible exceptions
                let exeption_class = exc.as_call_expr().unwrap();
                let class_name = exeption_class.func.as_name_expr().unwrap().id.to_string();

                let args = exeption_class
                    .args
                    .iter()
                    .map(|arg| r_e(arg, ctx))
                    .intersperse_with(|| Ok(",".to_string()))
                    .collect::<TResult<String>>()?;

                return Ok(format!("panic!(\"{class_name}({args})\")"));
            } else {
                return Ok(format!("panic!()"));
            }
        }
        Stmt::Try(StmtTry {
            range: _,
            body,
            handlers,
            orelse: _,
            finalbody: _,
        }) => {
            let body = body
                .iter()
                .map(|s| r_s(s, ctx))
                .intersperse_with(|| Ok(";\n".to_string()))
                .collect::<TResult<String>>()?;

            let handlers = handlers
                .iter()
                .map(|handler| {
                    let ExceptHandlerExceptHandler {
                        range: _,
                        type_: _,
                        name,
                        body,
                    } = handler.as_except_handler().unwrap();

                    let body = body
                        .iter()
                        .map(|s| r_s(s, ctx))
                        .collect::<TResult<String>>()
                        .unwrap();

                    let header = name
                        .as_ref()
                        .map(|name| name.to_string())
                        .unwrap_or("error_name".to_string());

                    format!("catch_it(|{header}| {{\n{body}\n}});")
                })
                .collect::<String>();

            // TODO
            // The else block lets you execute code when there is no error.
            // The finally block lets you execute code, regardless of the result of the try- and except blocks.
            // dbg!(orelse);
            // dbg!(finalbody);
            Ok(format!("try_it(|| {{{body}}});\n {handlers}"))
        }
        Stmt::TypeAlias(StmtTypeAlias {
            range: _,
            name,
            type_params: _,
            value,
        }) => {
            let value = r_annotation(value)?;
            let name = r_e(name, ctx)?;
            Ok(format!("type {name} = {value};\n"))
        }
        Stmt::Assert(StmtAssert {
            range: _,
            test,
            msg,
        }) => match msg {
            Some(msg) => Ok(format!(
                "assert!({test}, {msg});\n",
                test = r_e(test, ctx)?,
                msg = r_e(msg, ctx)?
            )),
            None => Ok(format!("assert!({test});\n", test = r_e(test, ctx)?)),
        },
        Stmt::Import(StmtImport { range: _, names }) => {
            for name in names {
                if name.name.as_str() == "math" {
                    ctx.imports.math_import_name = Some(
                        name.asname
                            .as_ref()
                            .unwrap_or_else(|| &name.name)
                            .to_string(),
                    );
                }
            }

            Ok(String::new())
        }
        Stmt::ImportFrom(StmtImportFrom {
            range: _,
            module,
            names,
            level,
        }) => {
            // https://docs.python.org/3/library/ast.html#ast.ImportFrom

            assert_eq!(
                level.unwrap().to_u32(),
                0,
                "only absolute imports supported"
            );

            let from = module.as_ref().unwrap().as_str();

            // we can do proper import tracking such that
            // import math as m; m.sin
            // and
            // import math as foo; foo.sin
            // both resolve to the same thing
            //
            // this is not an implementation of the actual python import logic
            // which is very complex but a simple approximation

            let map = match from {
                "math" => Some(&mut ctx.imports.math),
                "functools" => {
                    // TODO e.g. reduce
                    Some(&mut ctx.imports.functools)
                }
                "itertools" => {
                    // TODO e.g. reduce
                    Some(&mut ctx.imports.itertools)
                }
                _ => None,
            };

            if let Some(map) = map {
                for name in names {
                    let member = name.name.as_str();

                    let e = map.entry(member.to_string());
                    let m = e.or_default();

                    if let Some(rename) = &name.asname {
                        *m = Some(rename.to_string());
                    }
                }
            }

            Ok(String::new())
        }
        Stmt::Expr(StmtExpr { value, range: _ }) => Ok(r_e(value, ctx)?.to_string()),
        Stmt::Pass(_) => Ok("todo!()".to_string()),
        Stmt::Break(_) => Ok("break".to_string()),
        Stmt::Continue(_) => Ok("continue".to_string()),
        Stmt::Global(_) => {
            // TODO translate to static/ and or once_cell
            Err(todo_link!())
        }
        Stmt::AsyncFor(_) => Err(todo_link!()),
        Stmt::AsyncFunctionDef { .. } => Err(todo_link!()),
        Stmt::AsyncWith(_) => Err(todo_link!()),
        Stmt::Nonlocal(_) => Err(todo_link!()),
        Stmt::TryStar(_) => Err(todo_link!()),
        Stmt::With(_) => Err(todo_link!()),
    }
}

/// convert expression
fn r_e(node: &Expr, ctx: &mut Ctx) -> TResult<String> {
    match node {
        Expr::BoolOp(ExprBoolOp {
            op,
            values,
            range: _,
        }) => Ok(format!(
            "{l} {o} {r}",
            l = r_e(&values[0], ctx)?,
            o = r_bool(op),
            r = r_e(&values[1], ctx)?
        )),
        Expr::NamedExpr(ExprNamedExpr {
            target,
            value,
            range: _,
        }) => Ok(format!(
            "let Some({t}) = {v}",
            t = r_e(target, ctx)?,
            v = r_e(value, ctx)?
        )),
        Expr::BinOp(ExprBinOp {
            left,
            op,
            right,
            range: _,
        }) => {
            if op == &Operator::Pow {
                Ok(format!(
                    "{l}.powf({r})",
                    l = r_e(left, ctx)?,
                    r = r_e(right, ctx)?
                ))
            } else {
                Ok(format!(
                    "{l} {o} {r}",
                    l = r_e(left, ctx)?,
                    o = r_o(op)?,
                    r = r_e(right, ctx)?
                ))
            }
        }
        Expr::UnaryOp(ExprUnaryOp {
            op,
            operand,
            range: _,
        }) => Ok(format!(
            "{o}{r}",
            o = match op {
                UnaryOp::Invert => "~",
                UnaryOp::Not => "!",
                UnaryOp::UAdd => "+",
                UnaryOp::USub => "-",
            },
            r = r_e(operand, ctx)?
        )),
        Expr::Lambda(ExprLambda {
            args,
            body,
            range: _,
        }) => {
            if args.kwarg.is_some() || !args.kwonlyargs.is_empty() || !args.posonlyargs.is_empty() {
                return Err(todo_link!());
            }

            let args = args
                .clone()
                .into_python_arguments()
                .args
                .iter()
                .map(|a| a.arg.to_string())
                .intersperse_with(|| ", ".to_string())
                .collect::<String>();

            Ok(format!("|{args}| {{\n{b}\n}}", b = r_e(body, ctx)?))
        }
        Expr::IfExp(ExprIfExp {
            test,
            body,
            orelse,
            range: _,
        }) => Ok(format!(
            "if {t} {{ {b} }} else {{ {o} }}",
            t = r_e(test, ctx)?,
            b = r_e(body, ctx)?,
            o = r_e(orelse, ctx)?
        )),
        Expr::Dict(ExprDict {
            keys,
            values,
            range: _,
        }) => {
            if keys.is_empty() {
                return Ok("HashMap::new()".to_string());
            }

            Ok(format!(
                "[{k}].into_iter().zip([{v}].into_iter()).collect::<HashMap<_, _>>()",
                k = keys
                    .iter()
                    .map(|k| r_e(&k.clone().unwrap(), ctx))
                    .intersperse_with(|| Ok(", ".to_string()))
                    .collect::<TResult<String>>()?,
                v = values
                    .iter()
                    .map(|e| r_e(e, ctx))
                    .intersperse_with(|| Ok(", ".to_string()))
                    .collect::<TResult<String>>()?
            ))
        }
        Expr::Set(ExprSet { elts, range: _ }) => {
            if elts.is_empty() {
                return Ok("HashSet::new()".to_string());
            }

            Ok(format!(
                "[{e}].into_iter().collect::<HashSet<_>>()",
                e = elts
                    .iter()
                    .map(|e| r_e(e, ctx))
                    .intersperse_with(|| Ok(", ".to_string()))
                    .collect::<TResult<String>>()?
            ))
        }
        Expr::ListComp(ExprListComp {
            elt,
            generators,
            range: _,
        }) => gen_generator(generators, elt, Some("Vec::<_>"), ctx),
        Expr::SetComp(ExprSetComp {
            elt,
            generators,
            range: _,
        }) => gen_generator(generators, elt, Some("HashSet::<_,_>"), ctx),

        Expr::DictComp(ExprDictComp {
            range,
            key,
            value,
            generators,
        }) => {
            let body = Expr::Tuple(ExprTuple {
                range: *range,
                elts: vec![*key.clone(), *value.clone()],
                ctx: ExprContext::Load,
            });

            gen_generator(generators, &body, Some("HashMap::<_,_>"), ctx)
        }
        Expr::Compare(ExprCompare {
            left,
            ops,
            comparators,
            range: _,
        }) => {
            let mut s = vec![];
            for ((lhs, rhs), op) in std::iter::once(left.deref())
                .chain(comparators.iter())
                .map_windows(|[lhs, rhs]| (r_e(lhs, ctx).unwrap(), r_e(rhs, ctx).unwrap()))
                .zip(ops.iter())
            {
                if op == &CmpOp::In {
                    s.push(format!("{rhs}.into_iter().any(|v| v == {lhs})"));
                } else {
                    s.push(format!("{lhs} {op} {rhs}", op = r_c(op)?));
                }
            }

            Ok(s.iter()
                .cloned()
                .intersperse_with(|| "&&".to_string())
                .collect())
        }
        Expr::Call(ExprCall {
            func,
            args,
            keywords,
            range: _,
        }) => {
            let args: Vec<String> = args
                .iter()
                .map(|e| r_e(e, ctx))
                .collect::<TResult<Vec<_>>>()?;

            let function_name = r_e(func, ctx)?;

            let positional_args = args
                .iter()
                .cloned()
                .intersperse_with(|| ", ".to_string())
                .collect::<String>();

            let kwargs = keywords
                .iter()
                .map(|kw| {
                    format!(
                        "{name}: {value}",
                        name = kw.arg.clone().unwrap(),
                        value = r_e(&kw.value, ctx).unwrap()
                    )
                })
                .intersperse_with(|| ", ".to_string())
                .collect::<String>();

            let args_str = match (positional_args.is_empty(), kwargs.is_empty()) {
                (true, true) => String::new(),
                (true, false) => {
                    // TODO generate a struct with the FnName + "Params"
                    // to get named arguments in rust

                    format!("{function_name}Params {{\n{kwargs}\n}}")
                }
                (false, true) => positional_args,
                (false, false) => {
                    // args must come before kwargs
                    format!("{positional_args}, {function_name}Params {{\n{kwargs}\n}}")
                }
            };

            // support for Dataclass like classes
            // TOOD handle __init__ method as well
            if let Some(members) = ctx.get_class_members(&function_name) {
                // Class/ Struct Init
                let body = if keywords.is_empty() {
                    // only args
                    members
                        .iter()
                        .zip(args.iter())
                        .map(|(m, a)| format!("{m} : {a},"))
                        .intersperse("\n".to_string())
                        .collect::<String>()
                } else {
                    if !args.is_empty() {
                        // mixing keyword args and non keyword args here is probably an error
                        // double check the python spec on the exact sematics and be sure to translate them
                        return Err(todo_link!());
                    }

                    // only kwargs
                    keywords
                        .iter()
                        .map(|k| {
                            r_e(&k.value, ctx).map(|a| {
                                let m = &k.arg.as_deref().unwrap().to_string();
                                format!("{m} : {a},")
                            })
                        })
                        .intersperse(Ok("\n".to_string()))
                        .collect::<TResult<String>>()?
                };

                return Ok(format!("{function_name} {{\n{body}\n}}"));
            } else if function_name == "print" {
                let fmt = "\"{:?}\"";
                return Ok(format!("println!({fmt}, {args_str})"));
            } else if function_name == "enumerate" {
                return Ok(format!("{args_str}.iter().enumerate()"));
            } else if function_name == "zip" {
                let add_mapping = args.len() > 2;

                let args_str: String = args
                    .iter()
                    .cloned()
                    .intersperse_with(|| ", ".to_string())
                    .collect();

                let mut args = args.iter();
                let first = args.next().unwrap();

                let mut zips = format!("{first}.iter()");
                let mut maps = first.to_string();

                for arg in args {
                    zips = format!("{zips}.zip({arg}.iter())");
                    maps = format!("({maps}, {arg})");
                }

                if add_mapping {
                    zips = format!("{zips}.map(|{maps}| ({args_str}))")
                }

                return Ok(zips);
            } else if function_name == "str" {
                return Ok(format!("{args_str}.to_string()"));
            } else if function_name == "len" {
                return Ok(format!("{args_str}.len()"));
            } else if function_name == "sum" {
                return Ok(format!("{args_str}.iter().sum()"));
            } else if function_name == "int" || function_name == "float" {
                let f = r_annotation(func)?;
                return Ok(format!("(({args_str}) as {f})"));
            } else if function_name == "range" {
                // TODO handle start and step interval here in a more robust way
                // keyword args start stop step need to be handled, ideally
                // without any string based logic
                let args: Vec<_> = args_str.split(',').collect();
                match args.as_slice() {
                    [end] => return Ok(format!("(0..{end})")),
                    [start, end] => return Ok(format!("({start}..{end})")),
                    [_start, _step, _stop] => {
                        // start stop step
                        return Err(todo_link!());
                    }
                    _ => unreachable!(),
                }
            } else if let Some(module) = &ctx.imports.math_import_name
                && let Some(math_method) = function_name.strip_prefix(module)
            {
                let math_method = math_method.strip_prefix('.').unwrap();
                // TOOD automate this for all modules not just math
                // ctx.imports.called_math_fns.push(PyMath::from(math_method));
                ctx.imports.math.insert(math_method.to_string(), None);
                return Ok(format!("prelude::{math_method}({args_str})"));
            } else if let Some(json_method) = function_name.strip_prefix("json.") {
                if json_method == "loads" {
                    return Ok(format!("serde_json::from_string({args_str}).unwrap()"));
                } else if json_method == "dumps" {
                    return Ok(format!("serde_json::to_string({args_str}).unwrap()"));
                }
            } else if let Some(json_method) = function_name.strip_prefix("np.") {
                if json_method == "where" {
                    return Ok(format!("ndarray::azip(({args_str}), {{ TODO zip body }})"));
                } else {
                    // TODO add numpy polyfill like done for math
                    return Err(todo_link!());
                }
            }

            Ok(format!("{function_name}({args_str})"))
        }
        Expr::FormattedValue(ExprFormattedValue {
            value,
            conversion: _,
            format_spec: _,
            range: _,
        }) => Ok(r_e(value, ctx)?.to_string()),
        Expr::JoinedStr(ExprJoinedStr { values, range: _ }) => {
            let mut interpolations = vec![];
            let mut format_str = "format!(\"".to_string();
            for v in values {
                // if matches!(v, Expr::Constant { .. }) {
                //     let node = r_e(v, ctx)?;
                //     dbg!(&node);
                //     format_str.push_str(&node);
                // } else if matches!(v, Expr::FormattedValue { .. }) {
                interpolations.push(r_e(v, ctx)?);
                format_str.push_str("{:?}")
                // } else {
                // return Err(todo_link!());
                // }
            }
            format_str.push('\"');

            for interp in interpolations {
                format_str.push_str(&format!(", {interp}"));
            }

            format_str.push(')');
            Ok(format_str)
        }
        Expr::Constant(ExprConstant {
            value,
            kind: _,
            range: _,
        }) => match value {
            Constant::None => Ok("None".to_string()),
            Constant::Str(s) => Ok(format!("\"{}\"", s)),
            Constant::Bytes(bytes) => {
                // TODO read this in more detail
                // https://peps.python.org/pep-3112/

                // the rust docs are a bit sparse here as well
                // https://doc.rust-lang.org/reference/tokens.html#examples
                // https://doc.rust-lang.org/reference/tokens.html#byte-string-literals

                let bytes = bytes.iter().fold(String::new(), |mut output, b| {
                    let _ = write!(output, "\\x{:02x}", b);
                    output
                });

                Ok(format!("b\"{bytes}\""))
            }
            Constant::Bool(b) => {
                if *b {
                    Ok("true".to_string())
                } else {
                    Ok("false".to_string())
                }
            }
            Constant::Int(i) => Ok(i.to_string()),
            Constant::Tuple(_) => Err(todo_link!()),
            Constant::Float(f) => Ok(f.to_string()),
            Constant::Complex { .. } => Err(todo_link!()),
            Constant::Ellipsis => Err(todo_link!()),
        },
        Expr::Attribute(ExprAttribute {
            value,
            attr,
            ctx: _,
            range: _,
        }) => {
            let value = r_e(value, ctx)?;
            if ctx.has_enum(&value) {
                return Ok(format!("{value}::{attr}"));
            } else if attr == "append" {
                // attr is the function name which is being called
                return Ok(format!("{value}.push"));
            }

            Ok(format!("{value}.{attr}"))
        }
        Expr::Subscript(ExprSubscript {
            value,
            slice,
            ctx: _,
            range: _,
        }) => Ok(format!(
            "{v}[{s}]",
            v = r_e(value, ctx)?,
            s = r_e(slice, ctx)?
        )),
        Expr::Name(ExprName {
            id,
            ctx: _,
            range: _,
        }) => Ok(format!(
            "{prefix}{name}",
            prefix = if ctx.declare_var_mut { "mut " } else { "" },
            name = id
        )),
        Expr::List(ExprList {
            elts,
            ctx: _,
            range: _,
        }) => Ok(format!(
            "vec![{e}]",
            e = elts
                .iter()
                .map(|e| r_e(e, ctx))
                .intersperse_with(|| Ok(", ".to_string()))
                .collect::<TResult<String>>()?
        )),
        Expr::Tuple(ExprTuple {
            elts,
            ctx: _,
            range: _,
        }) => Ok(format!(
            "({e})",
            e = elts
                .iter()
                .map(|e| r_e(e, ctx))
                .intersperse_with(|| Ok(", ".to_string()))
                .collect::<TResult<String>>()?
        )),
        Expr::Slice(ExprSlice {
            lower,
            upper,
            step,
            range: _,
        }) => {
            let s = &step.as_deref().map(|e| r_e(e, ctx));

            let s = if let Some(Ok(s)) = s {
                format!(".iter().step_by({s}).collect::<Vec<_>>()")
            } else {
                "".to_string()
            };

            Ok(format!(
                "{l}..{u}{s}",
                l = lower
                    .as_deref()
                    .map(|e| r_e(e, ctx))
                    .unwrap_or(Ok("".to_string()))?,
                u = upper
                    .as_deref()
                    .map(|e| r_e(e, ctx))
                    .unwrap_or(Ok("".to_string()))?
            ))
        }
        Expr::GeneratorExp(ExprGeneratorExp {
            range: _,
            elt,
            generators,
        }) => {
            // https://peps.python.org/pep-0289/
            gen_generator(generators, elt, None, ctx)
        }
        Expr::Starred(ExprStarred {
            range: _,
            value,
            ctx: _ctx_inner,
        }) => {
            // https://docs.python.org/3/tutorial/controlflow.html#tut-unpacking-arguments
            let _value = r_e(value, ctx)?;
            Err(todo_link!())
        }
        // https://github.com/rust-lang/rfcs/pull/3513
        Expr::Await(_) => Err(todo_link!()),
        Expr::Yield(_) => Err(todo_link!()),
        Expr::YieldFrom(_) => Err(todo_link!()),
    }
}

fn gen_generator(
    generators: &[rustpython_parser::ast::Comprehension],
    elt: &Expr,
    collection_type: Option<&str>,
    ctx: &mut Ctx,
) -> TResult<String> {
    if generators.len() != 1 {
        // only one level nested for loops are supported
        return Err(todo_link!());
    }
    let g = &generators[0];
    let body = r_e(elt, ctx)?;

    let collect = if let Some(collection_type) = collection_type {
        format!(".collect::<{collection_type}>()")
    } else {
        String::new()
    };

    if let Some(ifs) = g.ifs.first() {
        let body = format!(
            "if {cond} {{ Some({body}) }} else {{ None }} ",
            cond = r_e(ifs, ctx)?
        );

        Ok(format!(
            "{gen}.into_iter().filter_map(|{target}| {{ {body} }}){collect}",
            gen = r_e(&g.iter, ctx)?,
            target = r_e(&g.target, ctx)?,
        ))
    } else {
        Ok(format!(
            "{gen}.into_iter().map(|{target}| {{ {body} }}){collect}",
            gen = r_e(&g.iter, ctx)?,
            target = r_e(&g.target, ctx)?,
        ))
    }
}

/// convert args
fn r_a(node: &ArgWithDefault, ctx: &mut Ctx) -> TResult<String> {
    let node = node.to_arg().0;
    if node.arg.as_str() == "self" {
        Ok("&self".to_string())
    } else {
        let t = &node
            .annotation
            .clone()
            .map(|e| r_annotation(&e))
            .unwrap_or(Ok("()".to_string()))?;

        if t.starts_with("numpy::") {
            ctx.numpy_array_args
                .push((node.arg.to_string(), t.to_string()));
        }

        Ok(format!("{n}: {t}", n = node.arg))
    }
}

fn r_annotation(e: &Expr) -> TResult<String> {
    match e {
        Expr::Constant(ExprConstant {
            range: _,
            value,
            kind: _,
        }) => Ok(value.clone().str().unwrap().to_owned()),
        Expr::Subscript(ExprSubscript {
            value,
            slice,
            ctx: _,
            range: _,
        }) => {
            let v = r_annotation(value)?;
            let s = r_annotation(slice)?;

            if v == "tuple" {
                Ok(format!("({s})"))
            } else if let Some(v) = v.strip_prefix("Np") {
                Ok(format!("numpy::Py{v}<{s}>"))
            } else {
                Ok(format!("{v}<{s}>"))
            }
        }
        Expr::Name(ExprName {
            id,
            ctx: _,
            range: _,
        }) => {
            // int str etc.

            // TODO maybe we could return a function here which could do
            // the correct wrapping etc.
            // this would make tuple vs generic more precise and would
            // allow better extensions
            Ok(match id.as_str() {
                "Dict" => "std::collections::HashMap".to_string(),
                "float" => "f64".to_string(),
                "int" => "isize".to_string(),
                "List" => "Vec".to_string(),
                "Optional" => "Option".to_string(),
                "str" => "String".to_string(),
                _ => id.to_string(),
            })
        }
        Expr::Tuple(ExprTuple {
            elts,
            ctx: _,
            range: _,
        }) => elts
            .iter()
            .map(r_annotation)
            .intersperse_with(|| Ok(", ".to_string()))
            .collect::<TResult<String>>(),
        Expr::Attribute(_) => Err(todo_link!()),
        Expr::Await(_) => Err(todo_link!()),
        Expr::BinOp(_) => Err(todo_link!()),
        Expr::BoolOp(_) => Err(todo_link!()),
        Expr::Call(_) => Err(todo_link!()),
        Expr::Compare(_) => Err(todo_link!()),
        Expr::Dict(_) => Err(todo_link!()),
        Expr::DictComp(_) => Err(todo_link!()),
        Expr::FormattedValue(_) => Err(todo_link!()),
        Expr::GeneratorExp(_) => Err(todo_link!()),
        Expr::IfExp(_) => Err(todo_link!()),
        Expr::JoinedStr(_) => Err(todo_link!()),
        Expr::Lambda(_) => Err(todo_link!()),
        Expr::List(_) => Err(todo_link!()),
        Expr::ListComp(_) => Err(todo_link!()),
        Expr::NamedExpr(_) => Err(todo_link!()),
        Expr::Set(_) => Err(todo_link!()),
        Expr::SetComp(_) => Err(todo_link!()),
        Expr::Slice(_) => Err(todo_link!()),
        Expr::Starred(_) => Err(todo_link!()),
        Expr::UnaryOp(_) => Err(todo_link!()),
        Expr::Yield(_) => Err(todo_link!()),
        Expr::YieldFrom(_) => Err(todo_link!()),
    }
}

/// convert binary operator
fn r_o(node: &Operator) -> TResult<&'static str> {
    match node {
        Operator::Add => Ok("+"),
        Operator::BitAnd => Ok("&"),
        Operator::BitOr => Ok("|"),
        Operator::BitXor => Ok("^"),
        Operator::Div => Ok("/"),
        Operator::LShift => Ok("<<"),
        Operator::Mod => Ok("%"),
        Operator::Mult => Ok("*"),
        Operator::RShift => Ok(">>"),
        Operator::Sub => Ok("-"),
        Operator::FloorDiv => Err(todo_link!()),
        Operator::MatMult => Err(todo_link!()),
        Operator::Pow => unreachable!(),
    }
}

/// convert boolean operation
fn r_bool(node: &BoolOp) -> &str {
    match node {
        BoolOp::And => "&&",
        BoolOp::Or => "||",
    }
}

/// convert comparison
fn r_c(node: &CmpOp) -> TResult<&str> {
    match node {
        CmpOp::Eq => Ok("=="),
        CmpOp::NotEq => Ok("!="),
        CmpOp::Lt => Ok("<"),
        CmpOp::LtE => Ok("<="),
        CmpOp::Gt => Ok(">"),
        CmpOp::GtE => Ok(">="),
        CmpOp::Is => Ok("=="),    // a bit of a hack
        CmpOp::IsNot => Ok("!="), // a bit of a hack
        CmpOp::In => unreachable!(),
        CmpOp::NotIn => Ok("!contains(TODO)"),
    }
}

fn r_p(node: &Pattern, ctx: &mut Ctx) -> TResult<String> {
    match node {
        Pattern::MatchValue(PatternMatchValue { range: _, value }) => r_e(value, ctx),
        Pattern::MatchAs(_) => Err(todo_link!()),
        Pattern::MatchClass(_) => Err(todo_link!()),
        Pattern::MatchMapping(_) => Err(todo_link!()),
        Pattern::MatchOr(_) => Err(todo_link!()),
        Pattern::MatchSequence(_) => Err(todo_link!()),
        Pattern::MatchSingleton(_) => Err(todo_link!()),
        Pattern::MatchStar(_) => Err(todo_link!()),
    }
}
