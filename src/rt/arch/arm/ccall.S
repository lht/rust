// Mark stack as non-executable
#if defined(__linux__) && defined(__ELF__)
.section	.note.GNU-stack, "", %progbits
#endif

#ifdef __ARM_EABI__
#define UNWIND
#else
#define UNWIND @
#endif

.text
.code 32
.arm
.align 2

.globl __morestack
#if !defined(__APPLE__)
.hidden __morestack
.type __morestack, %function
#endif
__morestack:
	UNWIND .fnstart
	UNWIND .save {r4, fp, lr}
	push {r4, fp, lr}
        UNWIND .movsp r4
	mov r4, sp
	mov sp, r2
	mov fp, sp
	blx r1
	mov sp, r4
	pop {r4, fp, lr}
	mov pc, lr
	UNWIND .fnend
