export adler32;
export mk_adler32;

/*
 * Defined in RFC1950:
 *
      ADLER32 (Adler-32 checksum)
         This contains a checksum value of the uncompressed data
         (excluding any dictionary data) computed according to Adler-32
         algorithm. This algorithm is a 32-bit extension and improvement
         of the Fletcher algorithm, used in the ITU-T X.224 / ISO 8073
         standard. See references [4] and [5] in Chapter 3, below)

         Adler-32 is composed of two sums accumulated per byte: s1 is
         the sum of all bytes, s2 is the sum of all s1 values. Both sums
         are done modulo 65521. s1 is initialized to 1, s2 to zero.  The
         Adler-32 checksum is stored as s2*65536 + s1 in most-
         significant-byte first (network) order.
*/
type adler32 =
    obj {
      fn input([u8]);
      fn result() -> u32;
      fn reset();
    };

const base: u32 = 65521u32;

fn mk_adler32() -> adler32 {
  type adler32_state = {
    mutable s1: u32,
    mutable s2: u32
  };

  obj adler32(st: adler32_state) {
    // a straight but inefficient implementation
    fn input(ibuf: [u8]) {
      let a = st.s1;
      let b = st.s2;
      let out = io::stdout();
      for i: u8 in ibuf {
        a = (a + (i as u32)) % base;
        b = (b + a) % base;
        out.write_str(#fmt["hello, a %u b %u \n", a as uint, b as uint]);
      }
      st.s1 = a;
      st.s2 = b;
    }

    fn reset() {
      st.s1 = 1u32;
      st.s2 = 0u32;
    }

    fn result() -> u32 {
      ret (st.s2 << 16u32) | st.s1;
    }
  }

  let st = {mutable s1: 1u32, mutable s2: 0u32};
  let adr = adler32(st);
  ret adr;
}
