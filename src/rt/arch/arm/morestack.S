// Mark stack as non-executable
#if defined(__linux__) && defined(__ELF__)
.section	.note.GNU-stack, "", %progbits
#endif

/* See i386/morestack.S for the lengthy, general explanation. */


#if defined(__APPLE__)
#define MORESTACK               ___morestack
#else
#define MORESTACK               __morestack
#endif

#ifdef __ARM_EABI__
#define UNWIND
#else
#define UNWIND @
#endif

.text
.code 32
.arm
#if defined(__APPLE__)
.align 2
#else
.align
#endif

.global rust_stack_exhausted
.global MORESTACK

// r4 and r5 are scratch registers for __morestack due to llvm
// ARMFrameLowering::adjustForSegmentedStacks() implementation.
#if !defined(__APPLE__)
.hidden MORESTACK
.type MORESTACK,%function
#endif
MORESTACK:
	UNWIND .fnstart
	// Save frame pointer and return address
	UNWIND .save {r4, r5}
	UNWIND .save {lr}
	UNWIND .save {r6, fp, lr}
	push {r6, fp, lr}

	UNWIND .movsp r6
	mov r6, sp
	UNWIND .setfp fp, sp, #4
	add fp, sp, #4

    // Save argument registers of the original function
    push {r0, r1, r2, r3, lr}

    // Create new stack
    bl rust_stack_exhausted@plt

    // the above function ensures that it never returns
    UNWIND .fnend
