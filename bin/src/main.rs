fn main() {
    use std::path::PathBuf;
    let flags = xflags::parse_or_exit! {
        /// input .py file
        required -i, --input input : PathBuf
        /// run rustfmt on the code
        optional --fmt
        /// output file
        optional -o, --output output : PathBuf
        // TODO 1
        // add option to generate cargo project
        // TODO 2
        // add option to run clippy on the code
    };

    let mut ctx = p2r::Ctx::default();
    let ast = std::fs::read_to_string(&flags.input).unwrap();

    let prg = p2r::p2r(&ast, &mut ctx).unwrap();

    std::fs::write(
        flags.output.unwrap_or(flags.input.with_extension("rs")),
        format!("fn py_main(){{\n{prg}\n}}"),
    )
    .unwrap();

    if flags.fmt {
        dbg!(std::process::Command::new("rustfmt")
            .arg("./out.rs")
            .spawn()
            .unwrap()
            .wait_with_output()
            .unwrap());
    }
}
