import syntax::{ast, codemap};
import driver::session;
import lib::llvm::llvm;
import util::common;
import std::{fs, str, option};

obj debug_info(builder: llvm::DIBuilderRef,
               loc: codemap::loc) {
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
    let di = debug_info(builder, loc);
    ret option::some(di);
}

fn as_buf(str: str) -> str::sbuf { str::as_buf(str, {|buf| buf }) }
