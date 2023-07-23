#![feature(iter_intersperse)]

use rustpython_parser::ast::{
    ArgWithDefault, BoolOp, CmpOp, Constant, Expr, ExprAttribute, ExprBinOp, ExprBoolOp, ExprCall,
    ExprCompare, ExprConstant, ExprDict, ExprFormattedValue, ExprIfExp, ExprJoinedStr, ExprLambda,
    ExprList, ExprListComp, ExprName, ExprNamedExpr, ExprSet, ExprSetComp, ExprSlice,
    ExprSubscript, ExprTuple, ExprUnaryOp, MatchCase, Mod, Operator, Pattern, PatternMatchValue,
    Stmt, StmtAnnAssign, StmtAssert, StmtAssign, StmtAugAssign, StmtClassDef, StmtExpr, StmtFor,
    StmtFunctionDef, StmtIf, StmtMatch, StmtReturn, StmtTypeAlias, StmtWhile, UnaryOp,
};

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
            "Err at File {file}, Line {line}",
            file = self.file,
            line = self.line
        ))
    }
}

impl std::error::Error for TranspileError {}

impl TranspileError {
    pub fn to_link(&self) -> String {
        format!(
            "please, go to: https:://github.com/benmkw/python2rust/blob/main/{file}#L{line} and help improve this crate",
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

#[cfg(test)]
mod test;

pub fn p2r(ast: &str, ctx: &mut Ctx) -> Result<String, ParseError> {
    let mut total = "fn main(){\n".to_string();

    let ast = rustpython_parser::parse(ast, rustpython_parser::Mode::Interactive, "./")
        .map_err(ParseError::ParseError)?;
    let body = match ast {
        Mod::Module(_) => Err(todo_link!()),
        Mod::Interactive(body) => Ok(body),
        Mod::Expression(_) => Err(todo_link!()),
        Mod::FunctionType(_) => Err(todo_link!()),
    }?;

    for b in body.body {
        // dbg!(&b);
        let rust = r_s(&b, ctx)?;
        total += &rust;
        if b.is_expr_stmt() {
            // TOOD this handling of expr is a bit hacky?
            total += ";\n"
        }
    }

    total += "}\n";

    Ok(if let Ok(t) = syn::parse_file(&total) {
        prettyplease::unparse(&t)
    } else {
        total
    })
}

static MATH: &str = "
/// polyfill for the python math module
mod math {
    // this can all be inlined into the callers using rust-analyzer

    #[inline(always)]
    pub fn sin(v : f64) -> f64 {
        v.sin()
    }

    #[inline(always)]
    pub fn cos(v : f64) -> f64 {
        v.sin()
    }

    #[inline(always)]
    pub fn pow(a : f64, b : f64) -> f64 {
        a.powf(b)
    }
}
";

#[derive(Debug, Clone, Default)]
pub struct Ctx {
    pub classes: Vec<(String, Vec<String>)>,
    pub enums: Vec<String>,
    pub in_enum: bool,
    pub declare_var_mut: bool,
}

impl Ctx {
    fn has_class(&self, class: &str) -> bool {
        self.classes.iter().any(|s| s.0.as_str() == class)
    }

    fn get_class(&self, class: &str) -> Option<&[String]> {
        self.classes
            .iter()
            .find(|s| s.0.as_str() == class)
            .map(|v| v.1.as_slice())
    }

    fn has_enum(&self, e: &str) -> bool {
        self.enums.iter().any(|s| s.as_str() == e)
    }
}

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
        }) => Ok(format!(
            "fn {n}({a}) -> {ret_type} {{\n{b}}}\n",
            n = name,
            a = &args
                .args
                .iter()
                .map(|a| r_a(a, ctx))
                .intersperse_with(|| Ok(", ".to_string()))
                .collect::<TResult<String>>()?,
            b = body
                .iter()
                .map(|s| r_s(s, ctx))
                .collect::<TResult<String>>()?,
            ret_type = match returns {
                Some(r) => r_e(r, ctx)?,
                None => "()".to_string(),
            }
        )),
        Stmt::AsyncFunctionDef { .. } => Err(todo_link!()),
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
                Ok(format!("return {v};\n", v = r_e(v, ctx)?))
            } else {
                Err(todo_link!())
            }
        }
        Stmt::Delete { .. } => Err(todo_link!()),
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
                annotation = r_e(annotation, ctx)?
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
        Stmt::AsyncFor(_) => Err(todo_link!()),
        Stmt::While(StmtWhile {
            range: _,
            test,
            body,
            orelse,
        }) => {
            if !orelse.is_empty() {
                return Err(todo_link!());
            }

            Ok(format!(
                "while {test} {{\n{body}\n}}\n",
                test = r_e(test, ctx)?,
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
            if orelse.len() >= 2 {
                // "this gets called recursively for elif chains, it seem"
                return Err(todo_link!());
            }

            let body = body
                .iter()
                .map(|s| r_s(s, ctx))
                .collect::<TResult<Vec<String>>>()?
                .iter()
                .cloned()
                .padded(";\n".to_string())
                .collect::<String>();

            let test = r_e(test, ctx)?;

            if orelse.is_empty() {
                Ok(format!("if {test} {{\n{body}\n}}\n"))
            } else {
                let orelse = orelse
                    .iter()
                    .map(|v| {
                        let e = r_s(v, ctx)?;
                        Ok(if !v.is_if_stmt() {
                            format!("{{\n{e};\n}}")
                        } else {
                            e
                        })
                    })
                    .collect::<TResult<String>>()?;

                Ok(format!("if {test} {{\n{body}\n}} else {orelse}\n"))
            }
        }
        Stmt::With(_) => Err(todo_link!()),
        Stmt::AsyncWith(_) => Err(todo_link!()),
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
        Stmt::Raise(_) => Err(todo_link!()),
        Stmt::Try(_) => Err(todo_link!()),
        Stmt::TypeAlias(StmtTypeAlias {
            range: _,
            name,
            type_params: _,
            value,
        }) => {
            // TODO replace r_e(value) with r_type(value)
            // which translates the python types to rust
            let value = r_e(value, ctx)?;
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
        Stmt::Import(i) => Ok(if i.names.iter().any(|v| v.name.as_str() == "math") {
            MATH.to_string()
        } else {
            String::new()
        }),
        Stmt::ImportFrom(_) => Ok(String::new()),
        Stmt::Global(_) => {
            // TODO translate to static/ and or once_cell
            Err(todo_link!())
        }
        Stmt::Nonlocal(_) => Err(todo_link!()),
        Stmt::Expr(StmtExpr { value, range: _ }) => Ok(r_e(value, ctx)?.to_string()),
        Stmt::Pass(_) => Ok("todo!()".to_string()),
        Stmt::Break(_) => Ok("break".to_string()),
        Stmt::Continue(_) => Ok("continue".to_string()),
        Stmt::TryStar(_) => Err(todo_link!()),
    }
}

fn r_e(node: &Expr, ctx: &mut Ctx) -> TResult<String> {
    match node {
        Expr::BoolOp(ExprBoolOp {
            op,
            values,
            range: _,
        }) => Ok(format!(
            "{l} {o} {r}",
            l = r_e(&values[0], ctx)?,
            o = r_bo(op),
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
            if args.args.len() >= 2 {
                Err(todo_link!())
            } else {
                Ok(format!(
                    "|{a}| {{\n{b}\n}}",
                    a = r_a(&args.args[0], ctx)?,
                    b = r_e(body, ctx)?
                ))
            }
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
        }) => gen_generator(generators, elt, "Vec::<_>", ctx),
        Expr::SetComp(ExprSetComp {
            elt,
            generators,
            range: _,
        }) => gen_generator(generators, elt, "HashSet::<_,_>", ctx),

        Expr::DictComp(_) => Err(todo_link!()),
        Expr::GeneratorExp(_) => Err(todo_link!()),
        Expr::Await(_) => Err(todo_link!()),
        Expr::Yield(_) => Err(todo_link!()),
        Expr::YieldFrom(_) => Err(todo_link!()),
        Expr::Compare(ExprCompare {
            left,
            ops,
            comparators,
            range: _,
        }) => {
            let mut s = String::new();
            let lhs = r_e(left, ctx)?;
            for (op, comparator) in ops.iter().zip(comparators.iter()) {
                let comp = r_e(comparator, ctx)?;
                if op == &CmpOp::In {
                    s.push_str(&format!("{comp}.into_iter().any(|v| v == {lhs})"));
                } else {
                    s.push_str(&format!("{lhs} {} {comp}", r_c(op)?));
                }
            }
            Ok(s)
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

            let args_str = args
                .iter()
                .cloned()
                .intersperse_with(|| ", ".to_string())
                .collect::<String>();

            let f = r_e(func, ctx)?;
            // support for Dataclass like classes
            // TOOD handle __init__ method as well
            if let Some(members) = ctx.get_class(&f) {
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

                return Ok(format!("{f} {{\n{body}\n}}"));
            } else if f == "print" {
                let fmt = "\"{:?}\"";
                return Ok(format!("println!({fmt}, {args_str})"));
            } else if f == "enumerate" {
                return Ok(format!("{args_str}.iter().enumerate()"));
            } else if f == "zip" {
                return Ok(args
                    .iter()
                    .cloned()
                    .reduce(|acc, x| format!("{acc}.zip({x}.iter())"))
                    .unwrap());
            } else if f == "str" {
                return Ok(format!("{args_str}.to_string()"));
            } else if f == "len" {
                return Ok(format!("{args_str}.len()"));
            } else if f == "sum" {
                return Ok(format!("{args_str}.iter().sum()"));
            } else if f == "isize" || f == "f64" {
                return Ok(format!("(({args_str}) as {f})"));
            } else if f == "range" {
                // TODO handle start and step interval somehow here
                return Ok(format!("(0..{args_str})"));
            } else if let Some(math_method) = f.strip_prefix("math.") {
                return Ok(format!("math::{math_method}({args_str})"));
            } else if let Some(json_method) = f.strip_prefix("json.") {
                if json_method == "loads" {
                    return Ok(format!("serde_json::from_string({args_str}).unwrap()"));
                } else if json_method == "dumps" {
                    return Ok(format!("serde_json::to_string({args_str}).unwrap()"));
                }
            } else if let Some(json_method) = f.strip_prefix("np.") {
                if json_method == "where" {
                    return Ok(format!("ndarray::azip(({args_str}), {{ TODO zip body }})"));
                } else {
                    // TODO add numpy polyfill like done for math
                    return Err(todo_link!());
                }
            }

            Ok(format!("{f}({args_str})"))
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
            Constant::Bytes(_b) => Err(todo_link!()),
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
        Expr::Starred(_) => Err(todo_link!()),
        Expr::Name(ExprName {
            id,
            ctx: _,
            range: _,
        }) => {
            let prefix = if ctx.declare_var_mut { "mut " } else { "" };

            let name = match id.as_str() {
                "int" => "isize".to_string(),
                "float" => "f64".to_string(),
                "str" => "String".to_string(),
                _ => id.to_string(),
            };

            Ok(format!("{prefix}{name}"))
        }
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
    }
}

fn gen_generator(
    generators: &Vec<rustpython_parser::ast::Comprehension>,
    elt: &Expr,
    collection_type: &str,
    ctx: &mut Ctx,
) -> TResult<String> {
    if generators.len() != 1 {
        // only one level nested for loops are supported
        return Err(todo_link!());
    }
    let g = &generators[0];
    let body = r_e(elt, ctx)?;
    if let Some(ifs) = g.ifs.first() {
        let body = format!(
            "if {cond} {{ Some({body}) }} else {{ None }} ",
            cond = r_e(ifs, ctx)?
        );

        Ok(format!(
            "{gen}.into_iter().filter_map(|{target}| {{ {body} }}).collect::<{collection_type}>()",
            gen = r_e(&g.iter, ctx)?,
            target = r_e(&g.target, ctx)?,
        ))
    } else {
        Ok(format!(
            "{gen}.into_iter().map(|{target}| {{ {body} }}).collect::<{collection_type}>()",
            gen = r_e(&g.iter, ctx)?,
            target = r_e(&g.target, ctx)?,
        ))
    }
}

fn r_a(node: &ArgWithDefault, ctx: &mut Ctx) -> TResult<String> {
    let node = node.to_arg().0;
    Ok(if node.arg.as_str() == "self" {
        "&self".to_string()
    } else {
        let t = &node
            .annotation
            .clone()
            .map(|e| r_e(&e, ctx))
            .unwrap_or(Ok("()".to_string()))?;

        format!("{n}: {t}", n = node.arg)
    })
}

fn r_o(node: &Operator) -> TResult<&'static str> {
    match node {
        Operator::Add => Ok("+"),
        Operator::Sub => Ok("-"),
        Operator::Mult => Ok("*"),
        Operator::MatMult => Err(todo_link!()),
        Operator::Div => Ok("/"),
        Operator::Mod => Ok("%"),
        Operator::Pow => unreachable!(),
        Operator::LShift => Ok("<<"),
        Operator::RShift => Ok(">>"),
        Operator::BitOr => Ok("|"),
        Operator::BitXor => Ok("^"),
        Operator::BitAnd => Ok("&"),
        Operator::FloorDiv => Err(todo_link!()),
    }
}

fn r_bo(node: &BoolOp) -> &str {
    match node {
        BoolOp::And => "&&",
        BoolOp::Or => "||",
    }
}

fn r_c(node: &CmpOp) -> TResult<&str> {
    match node {
        CmpOp::Eq => Ok("=="),
        CmpOp::NotEq => Ok("!="),
        CmpOp::Lt => Ok("<"),
        CmpOp::LtE => Ok("<="),
        CmpOp::Gt => Ok(">"),
        CmpOp::GtE => Ok(">="),
        CmpOp::Is => Err(todo_link!()),
        CmpOp::IsNot => Err(todo_link!()),
        CmpOp::In => unreachable!(),
        CmpOp::NotIn => Ok("!contains(TODO)"),
    }
}

fn r_p(node: &Pattern, ctx: &mut Ctx) -> TResult<String> {
    match node {
        Pattern::MatchValue(PatternMatchValue { range: _, value }) => r_e(value, ctx),
        Pattern::MatchSingleton(_) => Err(todo_link!()),
        Pattern::MatchSequence(_) => Err(todo_link!()),
        Pattern::MatchMapping(_) => Err(todo_link!()),
        Pattern::MatchClass(_) => Err(todo_link!()),
        Pattern::MatchStar(_) => Err(todo_link!()),
        Pattern::MatchAs(_) => Err(todo_link!()),
        Pattern::MatchOr(_) => Err(todo_link!()),
    }
}
