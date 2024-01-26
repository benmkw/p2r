from decorator import rust_decorator, NpReadonlyArrayDyn, f64, NpArrayDyn
import numpy as np


@rust_decorator
def comp(x: int) -> int:
    if x > 10:
        print("greater 10")
        return 0
    else:
        return 42

res = comp(3)
print(res)


# https://peps.python.org/pep-3107/
@rust_decorator
def math_arr_np(
    a: f64, x: NpReadonlyArrayDyn[f64], y: NpReadonlyArrayDyn[f64]
) -> NpArrayDyn[f64]:
    return a * x + y

arr_res = math_arr_np(a=3.0, x=np.array([1.0, 2.0, 3.0]), y=np.array([1.0, 2.0, 3.0]))
print(arr_res)

# TODO in place mutation
# support this using izip
# https://github.com/PyO3/rust-numpy/blob/32740b33ec55ef0b7ebec726288665837722841d/examples/simple/src/lib.rs#L103-L111
# @rust_decorator
# def math_arr(a : "numpy::PyReadwriteArray1::<f32>", b : "f32"):
#     a = a + b

# arr = np.array([1,2,3], 4.0)
# math_arr(arr)
# print(arr)
