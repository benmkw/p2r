use expect_test::expect;
use indoc::indoc;

// using https://github.com/rust-analyzer/expect-test
// set UPDATE_EXPECT and run `cargo test` for updating tests

#[test]
fn simple() {
    let actual = test_p2r("a = 4");
    let expected = expect![[r#"
        fn main() {
            let mut a = 4;
        }
    "#]];
    expected.assert_eq(&actual.to_string())
}

#[test]
fn list_comprehension() {
    let code = indoc! {"
        arr = [x for x in range(10)]
        s = {x for x in res if x in arr}
        "};

    let actual = test_p2r(code);
    let expected = expect![[r#"
        fn main() {
            let mut arr = (0..10).into_iter().map(|x| { x }).collect::<Vec<_>>();
            let mut s = res
                .into_iter()
                .filter_map(|x| {
                    if arr.into_iter().any(|v| *v == x) { Some(x) } else { None }
                })
                .collect::<HashSet<_, _>>();
        }
    "#]];
    expected.assert_eq(&actual.to_string())
}

#[test]
fn list_comprehension_with_if() {
    let actual = test_p2r("[x for x in range(10) if x % 2 == 0]");
    let expected = expect![[r#"
        fn main() {
            (0..10)
                .into_iter()
                .filter_map(|x| { if x % 2 == 0 { Some(x) } else { None } })
                .collect::<Vec<_>>();
        }
    "#]];
    expected.assert_eq(&actual.to_string())
}

#[test]
fn basic_for_and_while() {
    let code = indoc! {"
        a = 4
        x = 0
        for i in range(a):
            x += 1

        res = []
        while x > 0:
            x -= 1
            a = x**2
            res.append(a)
        "};

    let actual = test_p2r(code);
    let expected = expect![[r#"
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
    "#]];
    expected.assert_eq(&actual.to_string())
}

#[test]
fn basic_class() {
    let code = indoc! {"
        foo_i = Foo()
        class Foo:
            a : int
            b : int

            def member_fn(self, other : int) -> int:
                return 2 * self.a ** other

        foo_arg = Foo(1, 2*2)
        foo_kw = Foo(b=4-1, a=1)
        "};

    let actual = test_p2r(code);
    // TOOD fix this should be in impl
    let expected = expect![[r#"
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
    "#]];
    expected.assert_eq(&actual.to_string())
}

#[test]
fn basic_enum() {
    let code = indoc! {"
        foo_i = EFoo()
        class EFoo(Enum):
            A = 1
            B = 2

        a_inst = EFoo.A

        match a_inst:
            case EFoo.A: print(\"got an a\")
        "};

    let actual = test_p2r(code);
    let expected = expect![[r#"
        fn main() {
            let mut foo_i = EFoo();
            #[derive(Debug, Clone)]
            enum EFoo {
                A = 1,
                B = 2,
            }
            let mut a_inst = EFoo::A;
            match a_inst {
                EFoo::A => println!("{:?}", "got an a"),
            }
        }
    "#]];
    expected.assert_eq(&actual.to_string())
}

#[test]
fn tuple_fn() {
    let code = indoc! {"
        def t(x : int):
            return (3*x, 42)

        a,b = t(x)
        "};

    let actual = test_p2r(code);
    let expected = expect![[r#"
        fn main() {
            fn t(x: isize) -> () {
                return (3 * x, 42);
            }
            let (mut a, mut b) = t(x);
        }
    "#]];
    expected.assert_eq(&actual.to_string())
}

#[test]
fn test_lists_sets() {
    let code = indoc! {"
        a = []
        b = [1,2,3]
        is_in = 42 in b
        c = {}
        d = {1,2,3}
        e = {\"a\" : 1, \"b\" : 2, \"c\" : 3}
        "};

    let actual = test_p2r(code);
    let expected = expect![[r#"
        fn main() {
            let mut a = vec![];
            let mut b = vec![1, 2, 3];
            let mut is_in = b.into_iter().any(|v| *v == 42);
            let mut c = HashMap::new();
            let mut d = [1, 2, 3].into_iter().collect::<HashSet<_>>();
            let mut e = ["a", "b", "c"]
                .into_iter()
                .zip([1, 2, 3].into_iter())
                .collect::<HashMap<_, _>>();
        }
    "#]];
    expected.assert_eq(&actual.to_string())
}

#[test]
fn test_lambda() {
    let code = indoc! {"
        a = lambda x : x * 2
        b = a(3)
        "};

    let actual = test_p2r(code);
    let expected = expect![[r#"
        fn main() {
            let mut a = |x: ()| { x * 2 };
            let mut b = a(3);
        }
    "#]];
    expected.assert_eq(&actual.to_string())
}

#[test]
fn else_if() {
    let code = indoc! {"
        DEBUG = True
        res = []
        if DEBUG and len(res) > 2:
            a = 42
            print(1)
        elif DEBUG:
            print(2)
        elif DEBUG2:
            print(2)
        else:
            print(3)
        "};

    let actual = test_p2r(code);
    let expected = expect![[r#"
        fn main() {
            let mut DEBUG = true;
            let mut res = vec![];
            if DEBUG && res.len() > 2 {
                let mut a = 42;
                println!("{:?}", 1);
            } else if DEBUG {
                println!("{:?}", 2);
            } else if DEBUG2 {
                println!("{:?}", 2);
            } else {
                println!("{:?}", 3);
            }
        }
    "#]];
    expected.assert_eq(&actual.to_string())
}

#[test]
fn control() {
    let code = indoc! {"
        if TRUE:
            break
            continue
            pass
        assert 4 == 2+2, \"Oh no\"
        assert 3 == 2+1, \"Oh no\"
        "};

    let actual = test_p2r(code);
    let expected = expect![[r#"
        fn main() {
            if TRUE {
                break;
                continue;
                todo!();
            }
            assert!(4 == 2 + 2, "Oh no");
            assert!(3 == 2 + 1, "Oh no");
        }
    "#]];
    expected.assert_eq(&actual.to_string())
}

#[test]
fn test_math_polyfill() {
    let code = indoc! {"
        total = math.pow(math.sqrt(math.cos(math.sin(sum(res2)))), 32.1)
        total = sum(res2)
        "};

    let actual = test_p2r(code);
    let expected = expect![[r#"
        fn main() {
            let mut total = math::pow(math::sqrt(math::cos(math::sin(res2.iter().sum()))), 32.1);
            let mut total = res2.iter().sum();
        }
    "#]];
    expected.assert_eq(&actual.to_string())
}

#[test]
fn test_walrus() {
    // TOOD fix this
    let code = indoc! {"
        if x := map:
            print(f\"map ({map}) is {res[0] + 1}\")
        "};

    let actual = test_p2r(code);
    let expected = expect![[r#"
        fn main() {
            if let Some(x) = map {
                println!("{:?}", format!("{:?}{:?}{:?}{:?}", "map (", map, ") is ", res[0] + 1));
            }
        }
    "#]];
    expected.assert_eq(&actual.to_string())
}

#[test]
fn test_slicing() {
    let code = indoc! {"
        print(res2[:3], res2[1:2:30])
        "};

    let actual = test_p2r(code);
    let expected = expect![[r#"
        fn main() {
            println!("{:?}", res2[..3], res2[1..2.iter().step_by(30).collect::< Vec < _ >> ()]);
        }
    "#]];
    expected.assert_eq(&actual.to_string())
}

#[test]
fn test_json() {
    let code = indoc! {"
        foo_s = json.dumps(Foo())
        foo_instance = json.loads((foo_s))
        "};

    let actual = test_p2r(code);
    let expected = expect![[r#"
        fn main() {
            let mut foo_s = serde_json::to_string(Foo()).unwrap();
            let mut foo_instance = serde_json::from_string(foo_s).unwrap();
        }
    "#]];
    expected.assert_eq(&actual.to_string())
}

#[test]
fn test_zip() {
    // TODO this is not really good yet
    let code = indoc! {"
        a_arr = []
        b_arr = []
        c_arr = []
        for a,b,c in zip(a_arr, b_arr, c_arr):
            print(a, b, c)
        "};

    let actual = test_p2r(code);
    let expected = expect![[r#"
        fn main() {
            let mut a_arr = vec![];
            let mut b_arr = vec![];
            let mut c_arr = vec![];
            for (a, b, c) in a_arr.zip(b_arr.iter()).zip(c_arr.iter()) {
                println!("{:?}", a, b, c);
            }
        }
    "#]];
    expected.assert_eq(&actual.to_string())
}

#[test]
fn test_print_fmt() {
    // TODO this is not really good yet
    let code = indoc! {"
        a = 3
        b = f\"b is {2**2}\"
        print(f\"a is {a}\")
        "};

    let actual = test_p2r(code);
    let expected = expect![[r#"
        fn main() {
            let mut a = 3;
            let mut b = format!("{:?}{:?}", "b is ", 2.powf(2));
            println!("{:?}", format!("{:?}{:?}", "a is ", a));
        }
    "#]];
    expected.assert_eq(&actual.to_string())
}

#[test]
fn test_type_alias() {
    let code = indoc! {"
        type Point = tuple[float, float]
        "};

    let actual = test_p2r(code);
    let expected = expect![[r#"
        fn main(){
        type Point = tuple[(f64, f64)];
        }
    "#]];
    expected.assert_eq(&actual.to_string())
}

#[test]
fn test_casts() {
    let code = indoc! {"
        i = int(3.2)
        f = float(3 + 1)
        "};

    let actual = test_p2r(code);
    let expected = expect![[r#"
        fn main() {
            let mut i = ((3.2) as isize);
            let mut f = ((3 + 1) as f64);
        }
    "#]];
    expected.assert_eq(&actual.to_string())
}

#[test]
fn test_init() {
    // TODO improve this, using new instead on the rust side
    let code = indoc! {"
        class A:
            i : int
            f : float

        def __init__(self, i : int, f : float) -> A:
            self.i = i
            self.f = f
        "};

    let actual = test_p2r(code);
    let expected = expect![[r#"
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
    "#]];
    expected.assert_eq(&actual.to_string())
}

fn test_p2r(code: &str) -> String {
    crate::p2r(code, &mut crate::Ctx::default()).unwrap()
}
