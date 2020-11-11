#![feature(capture_disjoint_fields)]
//~^ WARNING the feature `capture_disjoint_fields` is incomplete
#![feature(rustc_attrs)]

enum Info {
    Point(i32, i32, String),
    Meta(String, Vec<(i32, i32)>)
}

fn multi_variant_enum() {
    let point = Info::Point(10, -10, "1".into());

    let vec = Vec::new();
    let meta = Info::Meta("meta".into(), vec);

    let c = #[rustc_capture_analysis]
    //~^ ERROR: attributes on expressions are experimental
    || {
        if let Info::Point(_, _, str) = point {
            //~^ Capturing point[] -> ImmBorrow
            //~| Capturing point[(2, 0)] -> ByValue
            //~| Min Capture point[] -> ByValue
            println!("{}", str);
        }

        if let Info::Meta(_, v) = meta {
            //~^ Capturing meta[] -> ImmBorrow
            //~| Capturing meta[(1, 1)] -> ByValue
            //~| Min Capture meta[] -> ByValue
            println!("{:?}", v);
        }
    };

    c();
}

enum SingleVariant {
    Point(i32, i32, String),
}

fn single_variant_enum() {
    let point = SingleVariant::Point(10, -10, "1".into());

    let c = #[rustc_capture_analysis]
    //~^ ERROR: attributes on expressions are experimental
    || {
    let SingleVariant::Point(_, _, str) = point;
        //~^ Capturing point[(2, 0)] -> ByValue
        //~| Min Capture point[(2, 0)] -> ByValue
        println!("{}", str);
    };

    c();
}

fn main() {}
