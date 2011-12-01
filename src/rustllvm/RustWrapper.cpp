//===- RustWrapper.cpp - Rust wrapper for core functions --------*- C++ -*-===
//
//                     The LLVM Compiler Infrastructure
//
// This file is distributed under the University of Illinois Open Source
// License. See LICENSE.TXT for details.
//
//===----------------------------------------------------------------------===
//
// This file defines alternate interfaces to core functions that are more
// readily callable by Rust's FFI.
//
//===----------------------------------------------------------------------===

#include "llvm/LLVMContext.h"
#include "llvm/Linker.h"
#include "llvm/PassManager.h"
#include "llvm/ADT/Triple.h"
#include "llvm/Assembly/Parser.h"
#include "llvm/Analysis/DIBuilder.h"
#include "llvm/Analysis/DebugInfo.h"
#include "llvm/Assembly/PrintModulePass.h"
#include "llvm/Support/FormattedStream.h"
#include "llvm/Support/Timer.h"
#include "llvm/Support/raw_ostream.h"
#include "llvm/Target/TargetMachine.h"
#include "llvm/Support/TargetSelect.h"
#include "llvm/Support/TargetRegistry.h"
#include "llvm/Support/SourceMgr.h"
#include "llvm/Target/TargetOptions.h"
#include "llvm/Support/Host.h"
#include "llvm/Metadata.h"
#include "llvm-c/Core.h"
#include "llvm-c/BitReader.h"
#include "llvm-c/Object.h"
#include <cstdlib>

using namespace llvm;

static const char *LLVMRustError;

extern "C" LLVMMemoryBufferRef
LLVMRustCreateMemoryBufferWithContentsOfFile(const char *Path) {
  LLVMMemoryBufferRef MemBuf = NULL;
  LLVMCreateMemoryBufferWithContentsOfFile(Path, &MemBuf,
    const_cast<char **>(&LLVMRustError));
  return MemBuf;
}

extern "C" const char *LLVMRustGetLastError(void) {
  return LLVMRustError;
}

extern "C" void LLVMAddBasicAliasAnalysisPass(LLVMPassManagerRef PM);

extern "C" void LLVMRustAddPrintModulePass(LLVMPassManagerRef PMR,
                                           LLVMModuleRef M,
                                           const char* path) {
  PassManager *PM = unwrap<PassManager>(PMR);
  std::string ErrorInfo;
  raw_fd_ostream OS(path, ErrorInfo, raw_fd_ostream::F_Binary);
  formatted_raw_ostream FOS(OS);
  PM->add(createPrintModulePass(&FOS));
  PM->run(*unwrap(M));
}

extern "C" bool LLVMLinkModules(LLVMModuleRef Dest, LLVMModuleRef Src) {
  static std::string err;

  // For some strange reason, unwrap() doesn't work here. "No matching
  // function" error.
  Module *DM = reinterpret_cast<Module *>(Dest);
  Module *SM = reinterpret_cast<Module *>(Src);
  if (Linker::LinkModules(DM, SM, Linker::DestroySource, &err)) {
    LLVMRustError = err.c_str();
    return false;
  }
  return true;
}

extern "C" void
LLVMRustWriteOutputFile(LLVMPassManagerRef PMR,
                        LLVMModuleRef M,
                        const char *triple,
                        const char *path,
                        TargetMachine::CodeGenFileType FileType,
                        CodeGenOpt::Level OptLevel) {

  // Set compilation options.
  llvm::NoFramePointerElim = true;

  InitializeAllTargets();
  InitializeAllTargetMCs();
  InitializeAllAsmPrinters();
  InitializeAllAsmParsers();
  std::string Err;
  const Target *TheTarget = TargetRegistry::lookupTarget(triple, Err);
  std::string FeaturesStr;
  std::string Trip(triple);
  std::string CPUStr = llvm::sys::getHostCPUName();
  TargetMachine *Target =
    TheTarget->createTargetMachine(Trip, CPUStr, FeaturesStr, Reloc::PIC_);
  bool NoVerify = false;
  PassManager *PM = unwrap<PassManager>(PMR);
  std::string ErrorInfo;
  raw_fd_ostream OS(path, ErrorInfo,
                    raw_fd_ostream::F_Binary);
  formatted_raw_ostream FOS(OS);

  bool foo = Target->addPassesToEmitFile(*PM, FOS, FileType, OptLevel,
                                         NoVerify);
  assert(!foo);
  (void)foo;
  PM->run(*unwrap(M));
  delete Target;
}

extern "C" LLVMModuleRef LLVMRustParseAssemblyFile(const char *Filename) {

  SMDiagnostic d;
  Module *m = ParseAssemblyFile(Filename, d, getGlobalContext());
  if (m) {
    return wrap(m);
  } else {
    LLVMRustError = d.getMessage().c_str();
    return NULL;
  }
}

extern "C" LLVMModuleRef LLVMRustParseBitcode(LLVMMemoryBufferRef MemBuf) {
  LLVMModuleRef M;
  return LLVMParseBitcode(MemBuf, &M, const_cast<char **>(&LLVMRustError))
         ? NULL : M;
}

extern "C" const char *LLVMRustGetHostTriple(void)
{
  static std::string str = llvm::sys::getHostTriple();
  return str.c_str();
}

extern "C" LLVMValueRef LLVMRustConstSmallInt(LLVMTypeRef IntTy, unsigned N,
                                              LLVMBool SignExtend) {
  return LLVMConstInt(IntTy, (unsigned long long)N, SignExtend);
}

extern "C" LLVMValueRef LLVMRustConstInt(LLVMTypeRef IntTy, 
					 unsigned N_hi,
					 unsigned N_lo,
					 LLVMBool SignExtend) {
  unsigned long long N = N_hi;
  N <<= 32;
  N |= N_lo;
  return LLVMConstInt(IntTy, N, SignExtend);
}

extern bool llvm::TimePassesIsEnabled;
extern "C" void LLVMRustEnableTimePasses() {
  TimePassesIsEnabled = true;
}

extern "C" void LLVMRustPrintPassTimings() {
  raw_fd_ostream OS (2, false); // stderr.
  TimerGroup::printAll(OS);
}

extern bool llvm::EnableSegmentedStacks;
extern "C" void LLVMRustEnableSegmentedStacks() {
  EnableSegmentedStacks = true;
}

extern "C" LLVMValueRef LLVMGetOrInsertFunction(LLVMModuleRef M,
                                                const char* Name,
                                                LLVMTypeRef FunctionTy) {
  return wrap(unwrap(M)->getOrInsertFunction(Name,
                                             unwrap<FunctionType>(FunctionTy)));
}


typedef struct LLVMOpaqueDIBuilder *LLVMDIBuilderRef;
typedef struct LLVMMDNode *LLVMMDNodeRef;
typedef struct LLVMDIDescriptor *LLVMDIDescriptorRef;
typedef struct LLVMDIFile *LLVMDIFileRef;
typedef struct LLVMDISubprogram *LLVMDISubprogramRef;
typedef struct LLVMDIType *LLVMDITypeRef;
typedef struct LLVMDIArray *LLVMDIArrayRef;

#define DEFINE_SIMPLE_CONVERSION_FUNCTIONS(ty, ref)   \
  inline ty *unwrap(ref P) {                          \
    return reinterpret_cast<ty*>(P);                  \
  }                                                   \
                                                      \
  inline ref wrap(const ty *P) {                      \
    return reinterpret_cast<ref>(const_cast<ty*>(P)); \
  }


DEFINE_SIMPLE_CONVERSION_FUNCTIONS(MDNode,           LLVMMDNodeRef)
DEFINE_SIMPLE_CONVERSION_FUNCTIONS(DIBuilder,        LLVMDIBuilderRef)
DEFINE_SIMPLE_CONVERSION_FUNCTIONS(DIDescriptor,     LLVMDIDescriptorRef)
DEFINE_SIMPLE_CONVERSION_FUNCTIONS(DISubprogram,     LLVMDISubprogramRef)
DEFINE_SIMPLE_CONVERSION_FUNCTIONS(DIType,           LLVMDITypeRef)
DEFINE_SIMPLE_CONVERSION_FUNCTIONS(DIFile,           LLVMDIFileRef)
DEFINE_SIMPLE_CONVERSION_FUNCTIONS(DIArray,          LLVMDIArrayRef)

#undef DEFINE_SIMPLE_CONVERSION_FUNCTIONS

extern "C" LLVMDIBuilderRef LLVMCreateDIBuilder(LLVMModuleRef M) {
  return wrap(new DIBuilder(*unwrap(M)));
}

extern "C" void LLVMDisposeDIBuilder(LLVMDIBuilderRef B) {
  delete unwrap(B);
}

extern "C" void LLVMFinalizeDIBuilder(LLVMDIBuilderRef B) {
  unwrap(B)->finalize();
}

extern "C" LLVMMDNodeRef LLVMDIGetCU(LLVMDIBuilderRef B) {
  return wrap(unwrap(B)->getCU());
}

extern "C"
void LLVMDIBuildCompileUnit(LLVMDIBuilderRef DB, unsigned Lang,
                            const char* Filename,
                            const char* Directory,
                            const char* Producer,
                            bool isOptimized,
                            const char* Flags,
                            unsigned RunTimeVer) {
  unwrap(DB)->createCompileUnit(Lang, Filename, Directory, Producer,
                                isOptimized, Flags, RunTimeVer);
}

extern "C" LLVMDIFileRef
LLVMDIBuildFile(LLVMDIBuilderRef DB,
                const char* Filename,
                const char* Directory) {
  DIFile f = unwrap(DB)->createFile(Filename, Directory);
  return wrap(&f);
}

extern "C" LLVMDISubprogramRef
LLVMDIBuildFunction(LLVMDIBuilderRef DB, LLVMMDNodeRef Scope,
                    const char* Name, const char* LinkageName,
                    LLVMDIFileRef File, unsigned LineNo,
                    LLVMDITypeRef Ty, bool isLocalToUnit,
                    bool isDefinition, unsigned Flags,
                    bool isOptimized, LLVMValueRef Fn,
                    LLVMMDNodeRef TParam, LLVMMDNodeRef Decl) {
  DIDescriptor d(unwrap(Scope));
  DISubprogram sp =
      unwrap(DB)->createFunction(d, Name,
                                 LinkageName,
                                 *unwrap(File), LineNo,
                                 *unwrap(Ty), isLocalToUnit,
                                 isDefinition,
                                 Flags,
                                 isOptimized,
                                 unwrap<Function>(Fn));
  /* FIXME
     unwrap(TParam),
     unwrap(Decl));
  */
  return wrap(&sp);
}

extern "C" LLVMDITypeRef
LLVMDIBuildBasicType(LLVMDIBuilderRef DB,
                     const char* Name, uint64_t SizeInBits,
                     uint64_t AlignInBits, unsigned Encoding) {
  DIType t = unwrap(DB)->createBasicType(Name, SizeInBits,
                                         AlignInBits, Encoding);
  return wrap(&t);
}

extern "C" LLVMDITypeRef
LLVMDIBuildSubroutineType(LLVMDIBuilderRef DB,
                          LLVMDIFileRef File,
                          LLVMDIArrayRef ParameterTypes) {
  DIType st = unwrap(DB)->createSubroutineType(*unwrap(File),
                                               *unwrap(ParameterTypes));
  return wrap(&st);
}

extern "C" LLVMDITypeRef
LLVMDIBuildMemberType(LLVMDIBuilderRef DB, LLVMDIDescriptorRef Scope,
                      const char* Name, LLVMDIFileRef File,
                      unsigned LineNo, uint64_t SizeInBits,
                      uint64_t AlignInBits, uint64_t OffsetInBits,
                      unsigned Flags, LLVMDITypeRef Ty) {
  DIType st = unwrap(DB)->createMemberType(*unwrap(Scope), Name, *unwrap(File),
                                           LineNo, SizeInBits, AlignInBits,
                                           OffsetInBits, Flags, *unwrap(Ty));
  return wrap(&st);
}

extern "C" LLVMDITypeRef
LLVMDIBuildStructType(LLVMDIBuilderRef DB, LLVMDIDescriptorRef Scope,
                      const char* Name, LLVMDIFileRef File,
                      unsigned LineNo, uint64_t SizeInBits,
                      uint64_t AlignInBits, unsigned Flags,
                      LLVMDIArrayRef Elements, unsigned RunTimeLang = 0) {
  DIType st = unwrap(DB)->createStructType(*unwrap(Scope), Name, *unwrap(File),
                                           LineNo, SizeInBits, AlignInBits,
                                           Flags, *unwrap(Elements),
                                           RunTimeLang);
  return wrap(&st);
}

extern "C" LLVMDIArrayRef
LLVMDIGetOrCreateArray(LLVMDIBuilderRef DB, LLVMValueRef *Indices,
                       unsigned NumIndices) {
  DIArray dt = unwrap(DB)->getOrCreateArray(
      makeArrayRef(unwrap<Value>(Indices, NumIndices), NumIndices));
  return wrap(&dt);
}
