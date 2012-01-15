// error-pattern:statement with non-unit type requires a semicolon

fn main() {
    for i in [0] {
        true
    }
}
