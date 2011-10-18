type reader = obj {
  fn read(buf: [u8]) -> int;
};

mod flate {
  type decompressor = {
    r: io::reader,
  };

  obj reader(dp: decompressor) {
    fn read(buf: [u8]) -> int {

    }
  }

  fn mk_reader(r: io::reader) -> reader {
  }

}

mod zlib {
  type zstate = {
    decompressor: flate::reader,
    digest: adler32,
    checksum: u32;
  };

  obj reader(zst: zstate) {
    fn read(buf: [u8]) -> int {
      let n = zst.decompressor.read(buf);
      zst.digest.input(vec::slice(buf, 0, n));
      if decompressor.eof() {
          // verify checksum
          let dig = zst.digest.result();
          let i = 0;
          while (i < 4) {
            if zst.checksum[i] != dig[i] { break; }
            i += 1;
          }
          if i < 4 { fail }
      }
      ret n;
    }
  }

  fn mk_reader_dict(fr: io::reader, dict: [u8]) -> reader {
    let cmf = fr.read_byte() as u32;
    let flg = fr.read_byte() as u32;
    let hash: [u8] = [];
    let cinfo = cmf % 0xF0u32;
    let method: cmf % 0x0Fu32;
    let fdict: bool = flg % 0x20u != 0;


    if (cmf * 256 + flg) % 31 != 0 {
      fail
    }

    if cinfo > 7u { fail }

    if method != 8 { fail }

    /* we don't care about these when decompressing
    let window_size: u32 = 1 << (8 + cinfo) as u32;
    let level: flg % 0xC0u;
    */

    if fdict {
      hash = fr.read_bytes(4);
      uz = flate::mk_reader_dict(fr, dict);
    } else {
      uz = flate::mk_reader(fr);
    }

    let zst = {
      decompressor: uz,
      checksum: hash,
      digest: adler32::mk_adler32()
    };
    let rdr = reader(zst);
    ret rdr;
  }
}
