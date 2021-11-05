use bpaf::{
    curry,
    info::Info,
    params::{command, short},
    run, Parser,
};
use std::str::FromStr;

#[derive(Debug, Clone)]
struct Foo {
    a: bool,
    b: bool,
    c: f64,
    cmd: Cmd,
}

#[derive(Debug, Clone)]
enum Cmd {
    Accelerate(bool),
    Break(bool),
}

fn speed() -> Parser<f64> {
    let m = short('m')
        .help("speed in MPH")
        .long("mph")
        .argument()
        .metavar("SPEED")
        .build()
        .parse(|s| f64::from_str(&s));
    let k = short('k')
        .long("kph")
        .help("speed in KPH")
        .argument()
        .metavar("SPEED")
        .build()
        .parse(|s| f64::from_str(&s).map(|s| s / 0.62));
    m.or_else(k)
}

pub fn main() {
    let info = Info::default().descr("this is a test").version("12");

    let fast = short('f')
        .long("fast")
        .switch()
        .help("Use faster acceleration")
        .build();
    let acc_parser = Parser::pure(Cmd::Accelerate).ap(fast);
    let acc = command(
        "accel",
        "command for acceleration",
        info.clone().descr("accelerating").for_parser(acc_parser),
    );

    let a = short('a')
        .long("AAAAA")
        .switch()
        .help("maps to a boolean, is optional")
        .build();
    let b = short('b')
        .req_switch()
        .help("also maps to a boolean but mandatory")
        .build();

    let mk = Parser::pure(curry!(|a, b, c, cmd| Foo { a, b, c, cmd }));
    let x = mk.ap(a).ap(b).ap(speed()).ap(acc);
    let y = info.for_parser(x);

    let xx = run(y);
    println!("{:?}", xx);
}