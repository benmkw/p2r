from typing import Generic, TypeVar
from p2r_decorator import rust
import importlib
import os
from hashlib import sha256 as hash

class f64:
    pass

class f32:
    pass

class uint8t:
    pass

class usize:
    pass

class isize:
    pass

T = TypeVar("T")
class NpReadonlyArrayDyn(Generic[T]):
    pass

class NpReadonlyArrayDyn(Generic[T]):
    pass

class NpArrayDyn(Generic[T]):
    pass


def write(filename: str, contents: str):
    os.makedirs(os.path.dirname(filename), exist_ok=True)
    with open(filename, "w") as f:
        f.write(contents)

# from https://docs.rs/itertools/latest/src/itertools/lib.rs.html#299-333
# izip = """
# macro_rules! izip {
#     ( @closure $p:pat => $tup:expr ) => { |$p| $tup };

#     ( @closure $p:pat => ( $($tup:tt)* ) , $_iter:expr $( , $tail:expr )* ) => {
#         $crate::izip!(@closure ($p, b) => ( $($tup)*, b ) $( , $tail )*)
#     };

#     ($first:expr $(,)*) => {$crate::__std_iter::IntoIterator::into_iter($first)};

#     ($first:expr, $second:expr $(,)*) => {
#         $crate::izip!($first).zip($second)};

#     ( $first:expr $( , $rest:expr )* $(,)* ) => {
#         $crate::izip!($first)
#             $(.zip($rest))*
#             .map($crate::izip!(@closure a => (a) $( , $rest )*))
#     };
# }
# """


# https://github.com/numba/numba/blob/5ef7c86f76a6e8cc90e9486487294e0c34024797/numba/core/decorators.py#L26
def rust_decorator(func):
    import inspect

    assert inspect.isfunction(func)
    source = inspect.getsource(func)
    name = func.__name__
    # id changes on each run and is thus bad for caching
    # mangled_name = f"{name}_{id(func)}"
    mangled_name = f"{name}_{hash(source.encode()).hexdigest()}"
    source = rust(source)
    pyo3_rs = f"""
#![forbid(unsafe_code)]

use numpy::IntoPyArray;

#[pyo3::pyfunction]
{source}
#[pyo3::pymodule]
fn {mangled_name}(_py: pyo3::Python<'_>, m: &pyo3::types::PyModule) -> pyo3::PyResult<()> {{
    m.add_function(pyo3::wrap_pyfunction!({name}, m)?)?;
    Ok(())
}}
    """

    pyo3_toml = f"""
[package]
name = "{mangled_name}"
version = "0.1.0"
edition = "2021"

[workspace]

[lib]
crate-type = ["cdylib"]

[dependencies]
pyo3 = {{ version = "0.20.2", features = ["extension-module"] }}
numpy = "0.20"
    """

    os.environ["CARGO_TARGET_DIR"] = "rust_cache/shared_target_cache"
    write(f"rust_cache/{mangled_name}/src/lib.rs", pyo3_rs)
    toml_path = f"rust_cache/{mangled_name}/Cargo.toml"
    write(toml_path, pyo3_toml)

    os.system(f"maturin develop --release --manifest-path={toml_path} > /dev/null 2>&1")
    # os.system(f"maturin develop --release --manifest-path={toml_path}")

    return getattr(importlib.import_module(mangled_name), name)

    # https://github.com/numba/numba/blob/5ef7c86f76a6e8cc90e9486487294e0c34024797/numba/core/dispatcher.py#L146-L169
    # https://github.com/numba/numba/blob/5ef7c86f76a6e8cc90e9486487294e0c34024797/numba/core/compiler.py#L736
