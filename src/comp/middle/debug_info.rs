import driver::session;
import lib::llvm::llvm;
import middle::ty::*;
import std::{fs, map, option, ptr, str, unsafe, vec};
import syntax::{ast, ast_util, codemap};
import util::{common, ppaux};

// Should have picked a value between (0x8000, 0xFFFF)
// But llvm::DIBuilder assert for range in [C89, D]
const DW_LANG_RUST: uint = 0x1u;


  // Encoding attribute values
const DW_ATE_address: uint = 0x01u;
const DW_ATE_boolean: uint = 0x02u;
const DW_ATE_complex_float: uint = 0x03u;
const DW_ATE_float: uint = 0x04u;
const DW_ATE_signed: uint = 0x05u;
const DW_ATE_signed_char: uint = 0x06u;
const DW_ATE_unsigned: uint = 0x07u;
const DW_ATE_unsigned_char: uint = 0x08u;
const DW_ATE_imaginary_float: uint = 0x09u;
const DW_ATE_packed_decimal: uint = 0x0au;
const DW_ATE_numeric_string: uint = 0x0bu;
const DW_ATE_edited: uint = 0x0cu;
const DW_ATE_signed_fixed: uint = 0x0du;
const DW_ATE_unsigned_fixed: uint = 0x0eu;
const DW_ATE_decimal_float: uint = 0x0fu;
const DW_ATE_UTF: uint = 0x10u;
const DW_ATE_lo_user: uint = 0x80u;
const DW_ATE_hi_user: uint = 0xffu;

fn as_buf(str: str) -> str::sbuf { str::as_buf(str, {|buf| buf }) }

resource DIBuilderRef_res(B: llvm::DIBuilderRef) {
    llvm::LLVMDisposeDIBuilder(B);
}

type dbg_ctxt = {
    builder: DIBuilderRef_res,
    tcx: ty::ctxt,
    is_opt: bool,
    cu: llvm::MDNodeRef,
    fcache: map::hashmap<str, llvm::DIFileRef>,
    tcache: map::hashmap<ty::t, llvm::DITypeRef>
};

fn di_cu(b: llvm::DIBuilderRef, sess: session::session, loc: codemap::loc) ->
    llvm::MDNodeRef {
    let path = fs::make_absolute(loc.filename);
    let filename = fs::basename(path);
    let dir = fs::dirname(path);
    let ver = common::version();
    let args = sess.get_args();
    llvm::LLVMDIBuildCompileUnit(
        b,
        DW_LANG_RUST,
        as_buf(filename),
        as_buf(dir),
        as_buf(ver),
        sess.get_opts().optimize != 0u,
        as_buf(args),
        0u      // dummy runtime version
    );
    ret llvm::LLVMDIGetCU(b);
}

fn ty(dcx: @dbg_ctxt, t: ty::t, loc: codemap::loc) -> llvm::DITypeRef {
    alt dcx.tcache.find(t) {
      option::some(dt) { ret dt; }
      option::none. { }
    }
    alt ty::struct(dcx.tcx, t) {
      ty_nil.  | ty_bool. | ty_int.  | ty_float. |
      ty_uint. | ty_machine(_) | ty_char. {
        ret ty_basic(dcx, t);
      }
      ty_rec(_) {
        ret ty_record(dcx, t, loc);
      }
    }
}

fn split_file_dir(path: str) -> (str, str) {
    ret (fs::dirname(path), fs::basename(path));
}

fn get_file(dcx: @dbg_ctxt, loc: codemap::loc) -> llvm::DIFileRef {
    let abs = fs::make_absolute(loc.filename);
    alt dcx.fcache.find(abs) {
      option::some(dt) { ret dt; }
      option::none. { }
    }
    let (dir, filename) = split_file_dir(abs);
    let file =
        llvm::LLVMDIBuildFile(*dcx.builder, as_buf(filename), as_buf(dir));
    dcx.fcache.insert(abs, file);
    ret file;
}

fn ty_field(dcx: @dbg_ctxt, t: ty::field, loc: codemap::loc) ->
    llvm::DITypeRef {
    let name = t.ident;
    let file = get_file(dcx, loc);
    let sty = ty::struct(dcx.tcx, t.mt.ty);
    let (sz, asz) = get_sizes(dcx, sty);
    let ft = ty(dcx, t.mt.ty, loc);
    let dt = llvm::LLVMDIBuildMemberType(*dcx.builder, noscope(),
                                         as_buf(name), file,
                                         loc.line, sz, asz, 0u64, 0u, ft);
    ret dt;
}

fn ty_record(dcx: @dbg_ctxt, t: ty::t, loc: codemap::loc) ->
    llvm::DITypeRef unsafe {
    import vec::unsafe;
    let els = [];
    alt ty::struct(dcx.tcx, t) {
      ty_rec(fds) {
        // FIXME: find loc of each field
        for fl: field in fds { els += [ty_field(dcx, fl, loc)]; }
      }
    }
    let els_ptr: *llvm::ValueRef =
        std::unsafe::reinterpret_cast(ptr::addr_of(els));
    let array = llvm::LLVMDIGetOrCreateArray(*dcx.builder, els_ptr,
                                             vec::len(els));
    let file = get_file(dcx, loc);
    // FIXME findout record size
    let sz = 0u64;
    let asz = 0u64;
    let dt = 
        llvm::LLVMDIBuildStructType(*dcx.builder, noscope(),
                                    as_buf(""), file,
                                    loc.line, sz, asz, 0u, array, 0u);
    dcx.tcache.insert(t, dt);
    ret dt;
}

fn ty_basic(dcx: @dbg_ctxt, t: ty::t) -> llvm::DITypeRef {
    let name = ppaux::ty_to_str(dcx.tcx, t);
    let sty = ty::struct(dcx.tcx, t);
    let enc = encode_of(sty);
    let (sz_bits, align_bits) = get_sizes(dcx, sty);
    let bt = llvm::LLVMDIBuildBasicType(*dcx.builder, as_buf(name),
                                        sz_bits, align_bits, enc);
    dcx.tcache.insert(t, bt);
    ret bt;
}

fn get_sizes(dcx: @dbg_ctxt, sty: ty::sty) -> (u64, u64) {
    let cfg = dcx.tcx.sess.get_targ_cfg();
    let sz_bits = size_of(cfg, sty);
    let align_bits = 32u64; // FIXME
    ret (sz_bits, align_bits);
}

fn size_of_ty_mach(tm: ast::ty_mach) -> u64 {
    ret alt tm {
      ast::ty_i8.  { 08u64 }
      ast::ty_i16. { 16u64 }
      ast::ty_i32. { 32u64 }
      ast::ty_i64. { 64u64 }
      ast::ty_u8.  { 08u64 }
      ast::ty_u16. { 18u64 }
      ast::ty_u32. { 32u64 }
      ast::ty_u64. { 64u64 }
      ast::ty_f32. { 32u64 }
      ast::ty_f64. { 64u64 }
    };
}

fn size_of(cfg: @session::config, sty: ty::sty) -> u64 {
    ret alt sty {
      ty_nil.   { 0u64 }
      ty_bool.  { 1u64 }
      ty_int. {
        size_of_ty_mach(cfg.int_type)
      }
      ty_float. {
        size_of_ty_mach(cfg.float_type)
      }
      ty_uint.  {
        size_of_ty_mach(cfg.uint_type)
      }
      ty_machine(tm) {
        alt tm {
          ast::ty_i8.  {  8u64 }
          ast::ty_i16. { 16u64 }
          ast::ty_i32. { 32u64 }
          ast::ty_i64. { 64u64 }
          ast::ty_u8.  {  8u64 }
          ast::ty_u16. { 16u64 }
          ast::ty_u32. { 32u64 }
          ast::ty_u64. { 64u64 }
          ast::ty_f32. { 32u64 }
          ast::ty_f64. { 64u64 }
        }
      }
      ty_char. { 32u64 }
      _ { fail "ty is not a basic type" }
    };
}

fn encode_of(sty: ty::sty) -> uint {
    ret alt sty {
      ty_nil.   { DW_ATE_unsigned }
      ty_bool.  { DW_ATE_unsigned }
      ty_int.   { DW_ATE_signed }
      ty_float. { DW_ATE_float }
      ty_uint.  { DW_ATE_unsigned }
      ty_machine(tm) {
        alt tm {
          ast::ty_i8.  { DW_ATE_signed }
          ast::ty_i16. { DW_ATE_signed }
          ast::ty_i32. { DW_ATE_signed }
          ast::ty_i64. { DW_ATE_signed }
          ast::ty_u8.  { DW_ATE_unsigned }
          ast::ty_u16. { DW_ATE_unsigned }
          ast::ty_u32. { DW_ATE_unsigned }
          ast::ty_u64. { DW_ATE_unsigned }
          ast::ty_f32. { DW_ATE_float }
          ast::ty_f64. { DW_ATE_float }
        }
      }
      ty_char. { DW_ATE_unsigned }
      _ { fail "ty is not a basic type" }
    };
}

fn mk_dbg_ctxt(tcx: ty::ctxt, llmod: llvm::ModuleRef, crate: @ast::crate) ->
    option::t<@dbg_ctxt> {
    if !tcx.sess.get_opts().debuginfo {
        ret option::none;
    }
    let builder = llvm::LLVMCreateDIBuilder(llmod);
    let loc = tcx.sess.lookup_pos(crate.span.lo);
    let type_cache =
        map::mk_hashmap::<ty::t, llvm::DITypeRef>(ty::hash_ty, ty::eq_ty);
    let dx = @{
        builder: DIBuilderRef_res(builder),
        tcx: tcx,
        is_opt: tcx.sess.get_opts().optimize != 0u,
        cu: di_cu(builder, tcx.sess, loc),
        fcache: map::new_str_hash::<llvm::DIFileRef>(),
        tcache: type_cache
    };
    ret option::some(dx);
}

fn noscope() -> llvm::DIDescriptorRef unsafe {
    const cnull: uint = 0u;
    ret unsafe::reinterpret_cast(ptr::addr_of(cnull));
}

fn dummy_fn_ty() -> llvm::DITypeRef unsafe {
    const cnull: uint = 0u;
    ret unsafe::reinterpret_cast(ptr::addr_of(cnull));
}

fn nil_mdnode() -> llvm::MDNodeRef unsafe {
    const cnull: uint = 0u;
    ret unsafe::reinterpret_cast(ptr::addr_of(cnull));
}

fn dummy_ty_params() -> llvm::MDNodeRef unsafe {
    const cnull: uint = 0u;
    ret unsafe::reinterpret_cast(ptr::addr_of(cnull));
}

fn emit_fn_start(dcx: @dbg_ctxt, link_name: str, name: str,
                 loc: codemap::loc,
                 local: bool, flags: uint,
                 llfn: llvm::ValueRef
                 //, ty_params: [ast::ty_param] //FIXME
                ) {
    let file = get_file(dcx, loc);
    llvm::LLVMDIBuildFunction(*dcx.builder, dcx.cu,
                              as_buf(name), as_buf(link_name),
                              file, loc.line, dummy_fn_ty(),
                              local,
                              true, // always defined
                              flags, dcx.is_opt, llfn,
                              dummy_ty_params(),
                              nil_mdnode());
}

fn finalize(dcx: @dbg_ctxt) {
    llvm::LLVMFinalizeDIBuilder(*dcx.builder);
}
