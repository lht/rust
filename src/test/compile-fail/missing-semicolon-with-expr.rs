// error-pattern:statement with non-unit type requires a semicolon
fn foo() {
  task::spawn {|| }
}

fn main() {
  foo();
}

