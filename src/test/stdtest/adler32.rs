use std;
import std::str;
import std::adler32;

#[test]
fn test() {
  fn check_buf(input: [u8], exp: u32) {
    let adler = adler32::mk_adler32();
    adler.input(input);
    let result = adler.result();
    assert (result == exp);
  }

  fn check_str(input: str, exp: u32) {
    check_buf(str::bytes(input), exp);
  }

  check_str("Wikipedia", 300286872u32);
}
