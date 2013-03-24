/*
 * converted from record_sp.S by arm-as-to-ios
 *  http://psellos.com/2012/08/2012.08.arm-as-to-ios-4.html
 */

/* Apple compatibility macros */
#if defined(SYS_macosx)
#define Glo(s) _##s
#define Loc(s) L##s
#if defined(MODEL_armv6)
        .machine  armv6
        .macro  .funtype
        .endm
        .macro  cbz
        cmp     $0, #0
        beq     $1
        .endm
#else
        .machine  armv7
        .thumb
        .macro  .funtype
        .thumb_func $0
        .endm
#endif
        .macro  .type
        .endm
        .macro  .size
        .endm
#else
#define Glo(s) s
#define Loc(s) .L##s
        .macro  .funtype symbol
        .type  \symbol, %function
        .endm
#endif
/* End Apple compatibility macros */

.text
.code 32
.arm
.align 2


.globl Glo(record_sp_limit)
.globl Glo(get_sp_limit)
.globl Glo(get_sp)

Glo(record_sp_limit):
	mov r3, r0
	ldr r0, Loc(Pmy_cpu)
	mov r1, #0
	mov r2, #0
    stmfd   sp!, {r3, r7}
    ldr     r7, Loc(P345)
    swi     #0
    ldmfd   sp!, {r3, r7}
    movs    r0, r0
	movmi	r0, #0

	ldr r1, Loc(Pmy_array)
	str r3, [r1, r0]
	mov pc, lr


Glo(get_sp_limit):
    ldr r0, Loc(Pmy_cpu)
	mov r1, #0
	mov r2, #0
    stmfd   sp!, {r4, r7}
    ldr     r7, Loc(P345)
    swi     #0
    ldmfd   sp!, {r4, r7}
    movs    r0, r0
	movmi	r0, #0
	mov r3, r0

	ldr r1, Loc(Pmy_array)
	ldr r0, [r1, r3]
	mov pc, lr


Glo(get_sp):
	mov r0, sp
	mov pc, lr
	
.data
my_cpu:	.long	0
.globl my_array
my_array:	
	.long	0
	.long	0
	.long	0
	.long	0
	.long	0
	.long	0
	.long	0
	.long	0	

/* Pool of addresses loaded into registers */

        .text
        .align 2
Loc(Pmy_cpu):
        .long my_cpu
Loc(P345):
        .long 345
Loc(Pmy_array):
        .long my_array
