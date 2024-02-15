// extracted from  p2r/src/test.rs using tests_to_md.py
const examples = {
  "simple": {
    "python": "a = 0"
  },
  "comprehensions": {
    "python": "arr = [x for x in range(5, 10)]\nset_res = {x for x in res if x in arr}\ndict_res = {k:2*v for k, v in zip(arr, arr)}"
  },
  "list_comprehension_with_if": {
    "python": "[x for x in range(10) if x % 2 == 0]"
  },
  "basic_for_and_while": {
    "python": "a = 4\nx = 0\nfor i in range(a):\n    x += 1\n\nres = []\nwhile x > 0:\n    x -= 1\n    a = x**2\n    res.append(a)"
  },
  "while_else": {
    "python": "res = []\nwhile x > 0:\n    x -= 1\nelse:\n    print(\\\"done\\\")"
  },
  "basic_class": {
    "python": "foo_i = Foo()\nclass Foo:\n    a : int\n    b : int\n\n    def member_fn(self, other : int) -> int:\n        return 2 * self.a ** other\n\nfoo_arg = Foo(1, 2*2)\nfoo_kw = Foo(b=4-1, a=1)"
  },
  "basic_enum": {
    "python": "def EnumOrFun():\n    pass\n\n# EnumOrFun is a function\nfoo_i = EnumOrFun()\n\nclass EnumOrFun(Enum):\n    A = 1\n    B = 2\n\n# EnumOrFun is a class\na_inst = EnumOrFun.A\n\nmatch a_inst:\n    case EnumOrFun.A: print(\\\"got an A\\\")\n    case EnumOrFun.B: print(\\\"got a B\\\")"
  },
  "tuple_fn": {
    "python": "def t(x : int) -> Optional[tuple[int, int]]:\n    if x < 3:\n        return (3*x, 42)\n    else:\n        return None\n\na,b = t(1)"
  },
  "lists_sets": {
    "python": "a = []\nb = [1,2,3]\nis_in = 42 in b\nc = {}\nd = {1,2,3}\ne = {\\\"a\\\" : 1, \\\"b\\\" : 2, \\\"c\\\" : 3}"
  },
  "lambda": {
    "python": "times_two = lambda x : x * 2\ntwice = times_two(3)\nadd = lambda x,y : x + y\nadd(3,4)"
  },
  "else_if": {
    "python": "DEBUG = True\nres = []\nif DEBUG and len(res) > 2:\n    a = 42\n    print(1)\nelif DEBUG:\n    print(2)\nelif DEBUG2:\n    print(3)\nelse:\n    print(\\\"inside else...\\\")\n    print(4)"
  },
  "control": {
    "python": "if TRUE:\n    break\n    continue\n    pass\nassert 4 == 2+2, \\\"Oh no\\\""
  },
  "math_polyfill": {
    "python": "import math\narr = [1.1,2.1,3.1]\ntotal = math.pow(math.sqrt(math.cos(math.sin(sum(arr)))), 32.1)\nprint(total)"
  },
  "walrus": {
    "python": "if x := map:\n    print(f\\\"map ({map}) is {res[0] + 1}\\\")"
  },
  "slicing": {
    "python": "res2 = [1,2,3,4]\nprint(res2[:3], res2[1:2:30])"
  },
  "json": {
    "python": "foo_s = json.dumps(Foo())\nfoo_instance = json.loads((foo_s))"
  },
  "zip": {
    "python": "a_arr = [1,2,3]\nb_arr = [4,5,6]\nc_arr = [7,8,9]\nfor a,b,c in zip(a_arr, b_arr, c_arr):\n    print(f\\\"a: {a} b: {b}, c:{c}\\\")"
  },
  "print_fmt": {
    "python": "a = 3\nb = f\\\"b is {2**2}\\\"\nprint(f\\\"a is {a}\\\")"
  },
  "type_alias": {
    "python": "type PointFloat2 = tuple[float, float]\ntype MaybeFloat = Optional[float]\ntype ListOptionTuple = List[Optional[tuple[float, str]]]\ntype DictIntStr = Dict[int, str]"
  },
  "casts": {
    "python": "i = int(3.2)\nf = float(3 + 1)\ns = str(1)"
  },
  "init": {
    "python": "class A:\n    i : int\n    f : float\n\ndef __init__(self, i : int, f : float) -> A:\n    self.i = i\n    self.f = f"
  },
  "delete": {
    "python": "a, b = 1, 2\ndel a, b"
  },
  "bytes": {
    "python": "hello_world = b'\\x7f\\x45\\x4c\\x46\\x01\\x01\\x01\\x00'"
  },
  "raise": {
    "python": "raise Exception('hello')"
  },
  "try_except": {
    "python": "try:\n    return 3 / 0\nexcept BaseException as e1:\n    # TODO handle ; here better\n    # in .rs code\n    # print(e)\n    val = 100\n    return val"
  },
  "kwargs": {
    "python": "def foo(x, a, b, c):\n    pass\n\nfoo(0, a=1, b=2, c=3)"
  },
  "compare": {
    "python": "a = 3 < 4\nb = 3 < 4 < 5"
  },
  "doc_comments": {
    "python": "def add(a, b):\n    \\\"\\\"\\\"Adds two numbers\n    Arguments:\n        a : Number\n        b : Number\n    \\\"\\\"\\\"\n    return a + b"
  },
  "duffinian": {
    "python": "# https://rosettacode.org/wiki/Duffinian_numbers#Python\n# with minor modifications (added typehints)\n\ndef factors(n : int) -> List[int]:\n    factors = []\n    for i in range(1, n + 1):\n       if n % i == 0:\n           factors.append(i)\n    return factors\ndef gcd(a : int, b : int) -> int:\n    while b != 0:\n        a, b = b, a % b\n    return a\nis_relively_prime = lambda a, b: gcd(a, b) == 1\nsigma_sum = lambda x: sum(factors(x))\nis_duffinian = lambda x: is_relively_prime(x, sigma_sum(x)) and len(factors(x)) > 2\ncount = 0\ni = 0\nwhile count < 50:\n    if is_duffinian(i):\n        print(i)\n        count += 1\n    i+=1\ncount2 = 0\nj = 0\nwhile count2 < 20:\n    if is_duffinian(j) and is_duffinian(j+1) and is_duffinian(j+2):\n        print(f\\\"({j},{j+1},{j+2})\\\")\n        count2 += 1\n        j+=3\n    j+=1"
  },
  "fizzbufzz": {
    "python": "# from https://rosettacode.org/wiki/FizzBuzz#Python\n\n# TODO\n# print (', '.join([(x%3<1)*'Fizz'+(x%5<1)*'Buzz' or str(x) for x in range(1,101)]))\n\n# TODO\n# print(*map(lambda n: 'Fizzbuzz '[(i):i+13] if (i := n**4%-15) > -14 else n, range(1,100)))\n\n[print('FizzBuzz') if i % 15 == 0 else print('Fizz') if i % 3 == 0 else print('Buzz') if i % 5 == 0 else print(i) for i in range(1,101)]"
  },
  "generator_expression": {
    "python": "sum(x*x for x in range(10))"
  },
  "basic_rust_decorator": {
    "python": "def math_arr_np(\n    a: f64, x: NpReadonlyArrayDyn[f64], y: NpReadonlyArrayDyn[f64]\n) -> NpArrayDyn[f64]:\n    return a * x + y"
  },
  "imports": {
    "python": "from math import sin as foo, cos, pi as mypi\nimport math as math\nsin_of_pi = math.abs(cos(foo(mypi)))\nprint(sin_of_pi)"
  }
};