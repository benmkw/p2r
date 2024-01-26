from textwrap import dedent
import json
# TODO generate godbolt links like in the web version

with open("test.rs") as f:
    contents = f.read()

    tests = contents.split("#[test]\nfn ")[1:]


    js_map = {}

    md = "<!-- extracted from  p2r/src/test.rs using tests_to_md.py -->\n"
    md += "# Testcases\n\n"

    for test in tests:
        name, rest = test.split("()", maxsplit=1)

        # input
        py_start_marker = "indoc! {\""
        py_end_marker = "\"};"
        py_code_start = rest.find(py_start_marker)
        py_code_end = rest.find(py_end_marker)

        py_code = dedent(rest[py_code_start + len(py_start_marker): py_code_end]).strip()

        # result
        rs_start_marker = "expect![[r#\""
        rs_end_marker = "\"#]];"
        rs_code_start = rest.find(rs_start_marker)
        rs_code_end = rest.find(rs_end_marker)

        rs_code = dedent(rest[rs_code_start + len(rs_start_marker): rs_code_end]).strip()

        md += f"## {name.replace('_', ' ')}\n\n"
        md += "Python:\n"
        md += "\n```python\n"
        md += py_code
        md += "\n```\n\n"

        md += "Rust:\n"
        md += "\n```rust\n"
        md += rs_code
        md += "\n```\n\n"

        js_map[name] = {
            "python": py_code,
            # this gets generated in the browser anyway and is not really needed
            # "rust": rs_code
        }

        with open ("../../p2rjs/testcases.js", "w") as f:
            f.write("// extracted from  p2r/src/test.rs using tests_to_md.py\n")
            f.write("const examples = ")
            json.dump(js_map, f, indent=2)
            f.write(';')

        with open("examples.md", "w") as f:
            f.write(md)
