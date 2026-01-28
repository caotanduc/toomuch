use toomuch::timeout::run;

fn main() {
    run(std::env::args().skip(1).collect());
}
