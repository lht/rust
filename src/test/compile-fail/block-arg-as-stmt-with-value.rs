// error-pattern:statement with non-unit type requires a semicolon

fn compute1() -> float {
    let v = [0f, 1f, 2f, 3f];

    vec::foldl(0f, v) { |x, y| x + y } - 10f
}

fn main() {
    let x = compute1();
    log(debug, x);
    assert(x == -4f);
}
