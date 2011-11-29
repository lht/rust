import driver::session;
import lib::llvm::llvm;
import middle::ty;
import std::{fs, map, str, option};
import syntax::{ast, ast_util, codemap};
import util::common;
import llvm::dwarf::*;

obj debug_info(builder: llvm::DIBuilderRef,
               loc: codemap::loc,
               file_cache: map::hashmap<str, llvm::DIFileRef>,
               type_cache: map::hashmap<ast::def_id, llvm::DITypeRef>) {

    // from functionType to DIType
    fn get_or_create_fn_type(fnty: llvm::TypeRef) -> llvm::DITypeRef {

    }

    fn lldity(t: ty::t) -> llvm::DITypeRef {
        alt t {
          ty_nil.  | ty_bool. | ty_int.  | ty_float. |
          ty_uint. | ty_machine(_) | ty_char. {
            lldity_basic(t)
          }
          ty_str. | ty_vec() {
            lldity_seq(t)
          }
          ty_box(tm) {
          }
          ty_tag(tid, subtys) {
          }
          ty_rec(fields) {
          }
          ty_tup(ts) {
          }

          ty_fn(proto, args, ret_ty, cf, constrs) {
          }
          ty_param(id, k) {
          }

          ty_uniq(tm) {
          }
          ty_ptr(tm) {
          }
          ty_type. {/* no-op */ }
          ty_native(_) {/* no-op */ }

          ty_native_fn(args, ret_ty) {
          }
          ty_obj(methods) {
          }
          ty_res(did, subty, tps) {
          }
          ty_var(id) {
          }
        }
    }

    fn get_or_create_type(t: ty::t, u: llvm::DIFileRef)
        -> llvm::DITypeRef {
        alt type_cache.find(ty) {
          some(t) { ret t; }
          none. { }
        }
        ret create_type(t, u);
    }

    fn basic_type_encoding(t: ty:t) -> uint {
        ret alt st {
          ty_nil.   { DW_ATE_unsigned }
          ty_bool.  { DW_ATE_unsigned }
          ty_int.   { DW_ATE_signed }
          ty_float. { DW_ATE_float }
          ty_uint.  { DW_ATE_unsigned }
          ty_machine(tm) {
            alt tm {
              ast::ty_i8.  { DW_ATE_signed  }
              ast::ty_i16. { DW_ATE_signed  }
              ast::ty_i32. { DW_ATE_signed  }
              ast::ty_i64. { DW_ATE_signed  }
              ast::ty_u8.  { DW_ATE_unsigned  }
              ast::ty_u16. { DW_ATE_unsigned }
              ast::ty_u32. { DW_ATE_unsigned }
              ast::ty_u64. { DW_ATE_unsigned }
              ast::ty_f32. { DW_ATE_float }
              ast::ty_f64. { DW_ATE_float  }
            }
          }
          ty_char. { DW_ATE_unsigned; }
          _. { fail "ty is not a basic type" }
        }
    }

    fn create_type(t: ty:t, f: llvm::DIFileRef) {
        // if type_has_static_size(ccx, t)
        let llty = trans::type_of(ccx, sp, t);
        let size = trans::llsize_of(llty);
        let align = trans::llalign_of(ccx, llty);
        let name = ppaux::ty_to_str(tcx, t);
        let encoding = get_basic_type_encoding(t);
        ret create_basic_type(name, size, align, encoding);
    }

    fn create_basic_type(name: str, sz_in_bits: u64,
                         align_in_bits: u64,
                         encoding: uint) -> llvm::DITypeRef {
        ret llvm::LLVMDIBuildBasicType(as_buf(name), sz_in_bits,
                                       align_in_bits, encoding);
    }

    fn create_file(path: str) -> llvm::DIFileRef {
        let filename = fs::basename(path);
        let dir = fs::dirname(path);
        let file =
            llvm::LLVMDIBuildFile(builder, as_buf(filename), as_buf(dir));
        file_cache.insert(path, file);
        ret file;
    }

    fn get_or_create_file(loc: codemap::loc) -> llvm::DIFileRef {
        let filename = fs::make_absolute(loc.filename);
        alt file_cache.find(filename) {
          option::some(df) { ret df; }
          option::none. { }
        }
        ret self.create_file(filename);
    }

    fn emit_fn_start(linkage_name: str, name: str, is_opt: bool,
                     llfty: llvm::TypeRef, llfn: llvm::ValueRef) {
        let file = self.get_or_create_file(loc);
        let fn_ty = self.get_or_create_fn_type(llfty);
        llvm::LLVMDIBuildFunction(builder,
                                  file as llvm::DIDescriptorRef,
                                  as_buf(name), as_buf(linkage_name),
                                  file, loc.line, fn_ty,
                                  false, // FIXME not-exported?
                                  true,  // is definition
                                  0u,     // FIXME flags prototyped
                                  is_opt,
                                  llfn as llvm::ValueRef,
                                  // template parameters
                                  0 as llvm::MDNodeRef,
                                  0 as llvm::MDNodeRef);
    }
}

// Should have picked a value between (0x8000, 0xFFFF)
// But llvm::DIBuilder assert for range in [C89, D]
const DW_LANG_RUST: uint = 0x1u;

fn mk_debug_info(sess: session::session, module: llvm::ModuleRef,
                 crate: @ast::crate) -> option::t<debug_info> {
    if !sess.get_opts().debuginfo {
        ret option::none;
    }
    // FIXME lht make this a resource
    let builder = llvm::LLVMCreateDIBuilder(module);
    let loc = sess.lookup_pos(crate.span.lo);
    let path = fs::make_absolute(loc.filename);
    let filename = fs::basename(path);
    let dir = fs::dirname(path);
    let ver = common::version();
    let args = sess.get_args();
    llvm::LLVMDIBuildCompileUnit(
        builder,
        DW_LANG_RUST,
        as_buf(filename),
        as_buf(dir),
        as_buf(ver),
        sess.get_opts().optimize != 0u,
        as_buf(args),
        0u      // dummy runtime version
    );
    let file_cache = map::new_str_hash::<llvm::DIFileRef>();
    let type_cache = common::new_def_hash<llvm::DITypeRef>();
    let di = debug_info(builder, loc, file_cache, type_cache);
    ret option::some(di);
}

fn as_buf(str: str) -> str::sbuf { str::as_buf(str, {|buf| buf }) }
