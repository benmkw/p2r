<!-- extracted from  p2r/src/test.rs using tests_to_md.py -->
# Testcases

## simple

Python:

```python
a = 0
```

Rust:

```rust
fn main() {
    let mut a = 0;
}
```

## comprehensions

Python:

```python
arr = [x for x in range(5, 10)]
set_res = {x for x in res if x in arr}
dict_res = {k:2*v for k, v in zip(arr, arr)}
```

Rust:

```rust
fn main() {
    let mut arr = (5..10).into_iter().map(|x| { x }).collect::<Vec<_>>();
    let mut set_res = res
        .into_iter()
        .filter_map(|x| { if arr.into_iter().any(|v| v == x) { Some(x) } else { None } })
        .collect::<HashSet<_, _>>();
    let mut dict_res = arr
        .iter()
        .zip(arr.iter())
        .into_iter()
        .map(|(k, v)| { (k, 2 * v) })
        .collect::<HashMap<_, _>>();
}
```

## list comprehension with if

Python:

```python
[x for x in range(10) if x % 2 == 0]
```

Rust:

```rust
fn main() {
    (0..10)
        .into_iter()
        .filter_map(|x| { if x % 2 == 0 { Some(x) } else { None } })
        .collect::<Vec<_>>();
}
```

## basic for and while

Python:

```python
a = 4
x = 0
for i in range(a):
    x += 1

res = []
while x > 0:
    x -= 1
    a = x**2
    res.append(a)
```

Rust:

```rust
fn main() {
    let mut a = 4;
    let mut x = 0;
    for i in (0..a) {
        x += 1;
    }
    let mut res = vec![];
    while x > 0 {
        x -= 1;
        let mut a = x.powf(2);
        res.push(a)
    }
}
```

## while else

Python:

```python
res = []
while x > 0:
    x -= 1
else:
    print(\"done\")
```

Rust:

```rust
fn main() {
    let mut res = vec![];
    while x > 0 {
        x -= 1;
    }
    if !(x > 0) {
        println!("{:?}", "done");
    }
}
```

## basic class

Python:

```python
foo_i = Foo()
class Foo:
    a : int
    b : int

    def member_fn(self, other : int) -> int:
        return 2 * self.a ** other

foo_arg = Foo(1, 2*2)
foo_kw = Foo(b=4-1, a=1)
```

Rust:

```rust
fn main() {
    let mut foo_i = Foo();
    #[derive(Debug, Clone)]
    struct Foo {
        a: isize,
        b: isize,
    }
    impl Foo {
        fn member_fn(&self, other: isize) -> isize {
            return 2 * self.a.powf(other);
        }
    }
    let mut foo_arg = Foo { a: 1, b: 2 * 2 };
    let mut foo_kw = Foo { b: 4 - 1, a: 1 };
}
```

## basic enum

Python:

```python
def EnumOrFun():
    pass

# EnumOrFun is a function
foo_i = EnumOrFun()

class EnumOrFun(Enum):
    A = 1
    B = 2

# EnumOrFun is a class
a_inst = EnumOrFun.A

match a_inst:
    case EnumOrFun.A: print(\"got an A\")
    case EnumOrFun.B: print(\"got a B\")
```

Rust:

```rust
fn main() {
    fn EnumOrFun() -> () {
        todo!()
    }
    let mut foo_i = EnumOrFun();
    #[derive(Debug, Clone)]
    enum EnumOrFun {
        A = 1,
        B = 2,
    }
    let mut a_inst = EnumOrFun::A;
    match a_inst {
        EnumOrFun::A => println!("{:?}", "got an A"),
        EnumOrFun::B => println!("{:?}", "got a B"),
    }
}
```

## tuple fn

Python:

```python
def t(x : int) -> Optional[tuple[int, int]]:
    if x < 3:
        return (3*x, 42)
    else:
        return None

a,b = t(1)
```

Rust:

```rust
fn main() {
    fn t(x: isize) -> Option<(isize, isize)> {
        if x < 3 {
            return ((3 * x, 42)).into();
        } else {
            return (None).into();
        }
    }
    let (mut a, mut b) = t(1);
}
```

## lists sets

Python:

```python
a = []
b = [1,2,3]
is_in = 42 in b
c = {}
d = {1,2,3}
e = {\"a\" : 1, \"b\" : 2, \"c\" : 3}
```

Rust:

```rust
fn main() {
    let mut a = vec![];
    let mut b = vec![1, 2, 3];
    let mut is_in = b.into_iter().any(|v| v == 42);
    let mut c = HashMap::new();
    let mut d = [1, 2, 3].into_iter().collect::<HashSet<_>>();
    let mut e = ["a", "b", "c"]
        .into_iter()
        .zip([1, 2, 3].into_iter())
        .collect::<HashMap<_, _>>();
}
```

## lambda

Python:

```python
times_two = lambda x : x * 2
twice = times_two(3)
add = lambda x,y : x + y
add(3,4)
```

Rust:

```rust
fn main() {
    let mut times_two = |x| { x * 2 };
    let mut twice = times_two(3);
    let mut add = |x, y| { x + y };
    add(3, 4);
}
```

## else if

Python:

```python
DEBUG = True
res = []
if DEBUG and len(res) > 2:
    a = 42
    print(1)
elif DEBUG:
    print(2)
elif DEBUG2:
    print(3)
else:
    print(\"inside else...\")
    print(4)
```

Rust:

```rust
fn main() {
    let mut DEBUG = true;
    let mut res = vec![];
    if DEBUG && res.len() > 2 {
        let mut a = 42;
        println!("{:?}", 1);
    } else {
        if DEBUG {
            println!("{:?}", 2);
        } else {
            if DEBUG2 {
                println!("{:?}", 3);
            } else {
                println!("{:?}", "inside else...");
                println!("{:?}", 4);
            };
        };
    }
}
```

## control

Python:

```python
if TRUE:
    break
    continue
    pass
assert 4 == 2+2, \"Oh no\"
```

Rust:

```rust
fn main() {
    if TRUE {
        break;
        continue;
        todo!();
    }
    assert!(4 == 2 + 2, "Oh no");
}
```

## math polyfill

Python:

```python
import math
arr = [1.1,2.1,3.1]
total = math.pow(math.sqrt(math.cos(math.sin(sum(arr)))), 32.1)
print(total)
```

Rust:

```rust
fn main() {
    let mut arr = vec![1.1, 2.1, 3.1];
    let mut total = prelude::pow(
        prelude::sqrt(prelude::cos(prelude::sin(arr.iter().sum()))),
        32.1,
    );
    println!("{:?}", total);
    use prelude::{cos, pow, sin, sqrt};
    mod prelude {
        #[inline(always)]
        pub fn cos(v: f64) -> f64 {
            v.cos()
        }
        #[inline(always)]
        pub fn pow(a: f64, b: f64) -> f64 {
            a.powf(b)
        }
        #[inline(always)]
        pub fn sin(v: f64) -> f64 {
            v.sin()
        }
        #[inline(always)]
        pub fn sqrt(a: f64) -> f64 {
            a.sqrt()
        }
    }
}
```

## walrus

Python:

```python
if x := map:
    print(f\"map ({map}) is {res[0] + 1}\")
```

Rust:

```rust
fn main() {
    if let Some(x) = map {
        println!("{:?}", format!("{:?}{:?}{:?}{:?}", "map (", map, ") is ", res[0] + 1));
    }
}
```

## slicing

Python:

```python
res2 = [1,2,3,4]
print(res2[:3], res2[1:2:30])
```

Rust:

```rust
fn main() {
    let mut res2 = vec![1, 2, 3, 4];
    println!("{:?}", res2[..3], res2[1..2.iter().step_by(30).collect::< Vec < _ >> ()]);
}
```

## json

Python:

```python
foo_s = json.dumps(Foo())
foo_instance = json.loads((foo_s))
```

Rust:

```rust
fn main() {
    let mut foo_s = serde_json::to_string(Foo()).unwrap();
    let mut foo_instance = serde_json::from_string(foo_s).unwrap();
}
```

## zip

Python:

```python
a_arr = [1,2,3]
b_arr = [4,5,6]
c_arr = [7,8,9]
for a,b,c in zip(a_arr, b_arr, c_arr):
    print(f\"a: {a} b: {b}, c:{c}\")
```

Rust:

```rust
fn main() {
    let mut a_arr = vec![1, 2, 3];
    let mut b_arr = vec![4, 5, 6];
    let mut c_arr = vec![7, 8, 9];
    for (a, b, c) in a_arr
        .iter()
        .zip(b_arr.iter())
        .zip(c_arr.iter())
        .map(|((a_arr, b_arr), c_arr)| (a_arr, b_arr, c_arr))
    {
        println!(
            "{:?}", format!("{:?}{:?}{:?}{:?}{:?}{:?}", "a: ", a, " b: ", b, ", c:", c)
        );
    }
}
```

## print fmt

Python:

```python
a = 3
b = f\"b is {2**2}\"
print(f\"a is {a}\")
```

Rust:

```rust
fn main() {
    let mut a = 3;
    let mut b = format!("{:?}{:?}", "b is ", 2.powf(2));
    println!("{:?}", format!("{:?}{:?}", "a is ", a));
}
```

## type alias

Python:

```python
type PointFloat2 = tuple[float, float]
type MaybeFloat = Optional[float]
type ListOptionTuple = List[Optional[tuple[float, str]]]
type DictIntStr = Dict[int, str]
```

Rust:

```rust
fn main() {
    type PointFloat2 = (f64, f64);
    type MaybeFloat = Option<f64>;
    type ListOptionTuple = Vec<Option<(f64, String)>>;
    type DictIntStr = std::collections::HashMap<isize, String>;
}
```

## casts

Python:

```python
i = int(3.2)
f = float(3 + 1)
s = str(1)
```

Rust:

```rust
fn main() {
    let mut i = ((3.2) as isize);
    let mut f = ((3 + 1) as f64);
    let mut s = 1.to_string();
}
```

## init

Python:

```python
class A:
    i : int
    f : float

def __init__(self, i : int, f : float) -> A:
    self.i = i
    self.f = f
```

Rust:

```rust
fn main() {
    #[derive(Debug, Clone)]
    struct A {
        i: isize,
        f: f64,
    }
    fn __init__(&self, i: isize, f: f64) -> A {
        self.i = i;
        self.f = f;
    }
}
```

## delete

Python:

```python
a, b = 1, 2
del a, b
```

Rust:

```rust
fn main() {
    let (mut a, mut b) = (1, 2);
    drop(a);
    drop(b)
}
```

## bytes

Python:

```python
hello_world = b'\x7f\x45\x4c\x46\x01\x01\x01\x00'
```

Rust:

```rust
fn main() {
    let mut hello_world = b"\x7f\x45\x4c\x46\x01\x01\x01\x00";
}
```

## raise

Python:

```python
raise Exception('hello')
```

Rust:

```rust
fn main() {
    panic!("Exception("hello ")")
}
```

## try except

Python:

```python
try:
    return 3 / 0
except BaseException as e1:
    # TODO handle ; here better
    # in .rs code
    # print(e)
    val = 100
    return val
```

Rust:

```rust
fn main() {
    try_it(|| {
        return 3 / 0;
    });
    catch_it(|e1| {
        let mut val = 100;
        return val;
    });
}
```

## kwargs

Python:

```python
def foo(x, a, b, c):
    pass

foo(0, a=1, b=2, c=3)
```

Rust:

```rust
fn main() {
    fn foo(x: (), a: (), b: (), c: ()) -> () {
        todo!()
    }
    foo(0, fooParams { a: 1, b: 2, c: 3 });
}
```

## compare

Python:

```python
a = 3 < 4
b = 3 < 4 < 5
```

Rust:

```rust
fn main() {
    let mut a = 3 < 4;
    let mut b = 3 < 4 && 4 < 5;
}
```

## doc comments

Python:

```python
def add(a, b):
    \"\"\"Adds two numbers
    Arguments:
        a : Number
        b : Number
    \"\"\"
    return a + b
```

Rust:

```rust
fn main() {
    /*! Adds two numbers
    Arguments:
        a : Number
        b : Number
     */
    fn add(a: (), b: ()) -> () {
        return a + b;
    }
}
```

## duffinian

Python:

```python
# https://rosettacode.org/wiki/Duffinian_numbers#Python
# with minor modifications (added typehints)

def factors(n : int) -> List[int]:
    factors = []
    for i in range(1, n + 1):
       if n % i == 0:
           factors.append(i)
    return factors
def gcd(a : int, b : int) -> int:
    while b != 0:
        a, b = b, a % b
    return a
is_relively_prime = lambda a, b: gcd(a, b) == 1
sigma_sum = lambda x: sum(factors(x))
is_duffinian = lambda x: is_relively_prime(x, sigma_sum(x)) and len(factors(x)) > 2
count = 0
i = 0
while count < 50:
    if is_duffinian(i):
        print(i)
        count += 1
    i+=1
count2 = 0
j = 0
while count2 < 20:
    if is_duffinian(j) and is_duffinian(j+1) and is_duffinian(j+2):
        print(f\"({j},{j+1},{j+2})\")
        count2 += 1
        j+=3
    j+=1
```

Rust:

```rust
fn main() {
    fn factors(n: isize) -> Vec<isize> {
        let mut factors = vec![];
        for i in (1..n + 1) {
            if n % i == 0 {
                factors.push(i);
            }
        }
        return factors;
    }
    fn gcd(a: isize, b: isize) -> isize {
        while b != 0 {
            let (mut a, mut b) = (b, a % b);
        }
        return a;
    }
    let mut is_relively_prime = |a, b| { gcd(a, b) == 1 };
    let mut sigma_sum = |x| { factors(x).iter().sum() };
    let mut is_duffinian = |x| {
        is_relively_prime(x, sigma_sum(x)) && factors(x).len() > 2
    };
    let mut count = 0;
    let mut i = 0;
    while count < 50 {
        if is_duffinian(i) {
            println!("{:?}", i);
            count += 1;
        }
        i += 1;
    }
    let mut count2 = 0;
    let mut j = 0;
    while count2 < 20 {
        if is_duffinian(j) && is_duffinian(j + 1) {
            println!(
                "{:?}", format!("{:?}{:?}{:?}{:?}{:?}{:?}{:?}", "(", j, ",", j + 1, ",",
                j + 2, ")")
            );
            count2 += 1;
            j += 3;
        }
        j += 1;
    }
}
```

## fizzbufzz

Python:

```python
# from https://rosettacode.org/wiki/FizzBuzz#Python

# TODO
# print (', '.join([(x%3<1)*'Fizz'+(x%5<1)*'Buzz' or str(x) for x in range(1,101)]))

# TODO
# print(*map(lambda n: 'Fizzbuzz '[(i):i+13] if (i := n**4%-15) > -14 else n, range(1,100)))

[print('FizzBuzz') if i % 15 == 0 else print('Fizz') if i % 3 == 0 else print('Buzz') if i % 5 == 0 else print(i) for i in range(1,101)]
```

Rust:

```rust
fn main() {
    (1..101)
        .into_iter()
        .map(|i| {
            if i % 15 == 0 {
                println!("{:?}", "FizzBuzz")
            } else {
                if i % 3 == 0 {
                    println!("{:?}", "Fizz")
                } else {
                    if i % 5 == 0 {
                        println!("{:?}", "Buzz")
                    } else {
                        println!("{:?}", i)
                    }
                }
            }
        })
        .collect::<Vec<_>>();
}
```

## generator expression

Python:

```python
sum(x*x for x in range(10))
```

Rust:

```rust
fn main() {
    (0..10).into_iter().map(|x| { x * x }).iter().sum();
}
```

## basic rust decorator

Python:

```python
def math_arr_np(
    a: f64, x: NpReadonlyArrayDyn[f64], y: NpReadonlyArrayDyn[f64]
) -> NpArrayDyn[f64]:
    return a * x + y
```

Rust:

```rust
fn main() {
    fn math_arr_np<'py>(
        py: pyo3::Python<'py>,
        a: f64,
        x: numpy::PyReadonlyArrayDyn<f64>,
        y: numpy::PyReadonlyArrayDyn<f64>,
    ) -> &'py numpy::PyArrayDyn<f64> {
        let x = x.as_array().to_owned();
        let y = y.as_array().to_owned();
        return (a * x + y).into_pyarray(py);
    }
}
```

## imports

Python:

```python
from math import sin as foo, cos, pi as mypi
import math as math
sin_of_pi = math.abs(cos(foo(mypi)))
print(sin_of_pi)
```

Rust:

```rust
fn main() {
    let mut sin_of_pi = prelude::abs(cos(foo(mypi)));
    println!("{:?}", sin_of_pi);
    use prelude::{abs, cos, pi as mypi, sin as foo};
    mod prelude {
        #[inline(always)]
        pub fn abs(a: f64) -> f64 {
            a.abs()
        }
        #[inline(always)]
        pub fn cos(v: f64) -> f64 {
            v.cos()
        }
        pub const pi: f64 = std::f64::consts::PI;
        #[inline(always)]
        pub fn sin(v: f64) -> f64 {
            v.sin()
        }
    }
}
```

