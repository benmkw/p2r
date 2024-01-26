fn main() {
    use std::path::PathBuf;
    let flags = xflags::parse_or_exit! {
        /// input .py filepath
        required -i, --input input : PathBuf
        /// run rustfmt on the code
        optional --fmt
        /// output .rs filepath (defaults to input path with .rs extension)
        optional -o, --output output : PathBuf
        // TODO 1
        // add option to generate cargo project
        // TODO 2
        // add option to run clippy on the code
        // TODO 3
        // add option to print result to stdout/ make it the default?
    };

    let prg = p2r::p2r(
        &std::fs::read_to_string(&flags.input).unwrap(),
        &mut p2r::Ctx::default(),
    )
    .unwrap();

    std::fs::write(
        flags.output.unwrap_or(flags.input.with_extension("rs")),
        p2r::fmt(&format!("fn main(){{{prg}}}")),
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
