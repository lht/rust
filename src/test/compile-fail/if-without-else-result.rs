// error-pattern:statement with non-unit type requires a semicolon

fn main() {
    let a = if true { true };
    log(debug, a);
}
