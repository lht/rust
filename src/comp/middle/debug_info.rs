import syntax::{ast, codemap};
import driver::session;
import lib::llvm::llvm;
import util::common;
import std::{fs, map, str, option};
import middle::trans_common::{fn_ctxt};

obj debug_info(builder: llvm::DIBuilderRef,
               loc: codemap::loc,
               file_cache: map::hashmap<str, llvm::DIFileRef>) {

    // from functionType to DIType
    fn get_or_create_fn_type(fnty: llvm::ValueRef) -> llvm::DITypeRef {
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
                     llfn: llvm::ValueRef) {
        let file = self.get_or_create_file(loc);
        let fn_ty = self.get_or_create_fn_type();
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
    let di = debug_info(builder, loc, file_cache);
    ret option::some(di);
}

fn as_buf(str: str) -> str::sbuf { str::as_buf(str, {|buf| buf }) }

fn emit_fn_start(di: debug_info, fcx: @fn_ctxt) {
    let linkage_name = fcx.lcx.ccx.item_symbols.get(fcx.id);
    let name = str::connect(fcx.lcx.path, "::");
    let is_opt = fcx.lcx.ccx.sess.get_opts().optimize != 0u;
    di.emit_fn_start(linkage_name, name, is_opt, fcx.llfn);
}
