fn py_main(){
fn foo() {
    /// polyfill for the python math module
    mod math {
        #[inline(always)]
        pub fn sin(v: f64) -> f64 {
            v.sin()
        }
        #[inline(always)]
        pub fn cos(v: f64) -> f64 {
            v.sin()
        }
        #[inline(always)]
        pub fn pow(a: f64, b: f64) -> f64 {
            a.powf(b)
        }
    }
    #[derive(Debug, Clone)]
    struct Foo {
        a: isize,
        b: f64,
    }
    impl Foo {
        fn member_fn(&self, other: isize) -> isize {
            return 0 + other;
        }
        fn get_a(&self) -> () {
            return self.a;
        }
    }
    fn foo_math(i: isize) -> () {
        let x = i + 30;
        return (x * 2 + 3.powf(2), 42);
    }
    let foo_s = serde_json::to_string(Foo::new()).unwrap();
    let foo_instance = serde_json::from_string(foo_s).unwrap();
    let (a, b) = foo_math(3);
    println!(a);
    #[derive(Debug, Clone)]
    enum E_ABC {
        A = 1,
        B = 2,
        C = 3,
    }
    let e_inst = E_ABC::A;
    match e_inst {
        E_ABC::A => println!("A"),
        E_ABC::B => println!("H"),
        E_ABC::C => println!("C"),
    }
    let res = vec![];
    for i in 0..10 {
        res.push(i * 2);
        res.push(i);
    }
    let if_expr = if true { res[0].powf(3) } else { -4 };
    for (i, x) in res.iter().enumerate() {
        println!(i, x);
    }
    let z = |val: ()| { String(val) };
    let a = 0;
    while a < 10 {
        a += 1;
        break;
    }
    assert!(3 == 2 + 1, "Oh no");
    let DEBUG = true;
    if DEBUG && res.len() > 2 {
        let a = 42;
        println!("debugging");
    } else if DEBUG {
        println!("DEBUG");
    } else if DEBUG2 {
        println!("DEBUG2");
    } else {
        println!("else...");
    }
    for a in vec!["a", "b", "c"] {
        continue;
    }
    let res2 = res.iter().map(|x| { x * 2 }).collect::<Vec<_>>();
    println!(res2[..3], res2[1..2.iter().step_by(30).collect::< Vec < _ >> ()]);
    if res2.iter().contains(42) {
        println!("found 42!");
    }
    let map = ["some", "other"]
        .into_iter()
        .zip([1, 3].into_iter())
        .collect::<HashMap<_, _>>();
    let empty_map = HashMap::new();
    let s = [1, 2, 3, 3].into_iter().collect::<HashSet<_>>();
    let s = res
        .iter()
        .filter_map(|x| { if map.iter().contains(x) { Some(x) } else { None } })
        .collect::<HashSet<_, _>>();
    let map = HashMap::new();
    let total = res2.iter().sum();
    let total = math::pow(math::sqrt(math::cos(math::sin(res2.iter().sum()))), 32.1);
    if let Some(x) = map {
        println!("map (" {} ") is " {}, map, res[0] + 1);
    }
    let res3 = 0..10
        .iter()
        .filter_map(|x| { if x % 2 == 0 { Some(x) } else { None } })
        .collect::<Vec<_>>();
}

}