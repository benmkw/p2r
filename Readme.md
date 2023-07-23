# Python To Rust

## Key Features

- Minimal converter for Python to Rust code
- Automates repetitive adjustments like adding parentheses and semicolons
- Supports porting of math formulas, reducing the risk of errors
- Highly hackable and adaptable to new codebases
- Provides side-by-side comparison of Python and Rust code using Godbolt in the web UI
- Shows python programmers roughly equivalent (mostly) idiomatic rust code

The [web UI](https://benmkw.github.io/p2r/) has an option to show the python and rust code side by side in godbolt.

For supported Python features, please refer to `p2r/src/test.rs`.

This project aims to facilitate the process of converting Python code to Rust, making it easier to port existing Python projects to Rust. As Python is a dynamic language, some constructs may not translate seamlessly and efficiently. However, this converter automates many repetitive adjustments, such as adding parentheses and semicolons, reducing the manual effort required for the migration.

The genereated code can have some issues which can be automatically fixed by clippy like unneeded parantheses. Simple generated code which conveys the (assumed) intention is prefered over complicated code which mimics python 100% (otherwise basically everything would be to be translated to HashMap lookups and we we just write a slow python interpreter, basically). This tool should thus be seen as an aid for porting code rather than something like mojo etc. which aims to make a superset of python faster.

## CLI

in `bin` folder

```
OPTIONS:
    -i, --input <input>
      input .py file

    --fmt
      run rustfmt on the code

    -o, --output <output>
      output file

    -h, --help
      Prints help information.
```

## How to run the web ui in p2rjs

in `p2rjs` folder

`wasm-pack build --target web`
then do
`python3 -m http.server`

## TODO

This project is a work in progress, and there are several enhancements and features planned for the future.

- Add comments and possibly whitespace (the python parser does not currently forward them).
- Improve translation of zip by generating nested tuples for iterator.
- Support variable assignment with types (a relatively straightforward addition).
- Infer return types using heuristics or a Python static analyzer as a basis.
- Generate Python code and put it in comments for unknown code instead of aborting.
- Improve format string handling and generate more idiomatic Rust code.
- Support common numpy operations and add ndarray prelude for it.
- Handle `__init__` with `new`, not just dataclass-like classes.
- Improve the web UI with dropdown examples scraped from tests.
- Use ACE as the editor instead of a textarea and show errors at positions in the file in the web UI.
- Generate a whole cargo project for CLI users.

## Related Work

- [pyrs](https://github.com/konchunas/pyrs)
- [py2many](https://github.com/py2many/py2many)

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
