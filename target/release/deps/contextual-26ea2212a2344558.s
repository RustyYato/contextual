	.text
	.def	 @feat.00;
	.scl	3;
	.type	0;
	.endef
	.globl	@feat.00
.set @feat.00, 0
	.intel_syntax noprefix
	.file	"contextual.8le93y83-cgu.0"
	.def	 _ZN10contextual3asm17h7a821eeb655f3e5dE;
	.scl	2;
	.type	32;
	.endef
	.section	.text,"xr",one_only,_ZN10contextual3asm17h7a821eeb655f3e5dE
	.globl	_ZN10contextual3asm17h7a821eeb655f3e5dE
	.p2align	4, 0x90
_ZN10contextual3asm17h7a821eeb655f3e5dE:
.seh_proc _ZN10contextual3asm17h7a821eeb655f3e5dE
	push	rsi
	.seh_pushreg rsi
	push	rdi
	.seh_pushreg rdi
	sub	rsp, 40
	.seh_stackalloc 40
	.seh_endprologue
	mov	rax, qword ptr [rcx + 40]
	mov	rdi, qword ptr [rcx + 32]
	mov	qword ptr [rsp + 32], rdi
	mov	rdx, rax
	or	rdx, rdi
	shr	rdx, 32
	je	.LBB0_1
	xor	edx, edx
	div	rdi
	mov	r9, qword ptr [rcx + 8]
	mov	r8, qword ptr [rcx + 24]
	test	r8, r8
	jne	.LBB0_4
	jmp	.LBB0_11
.LBB0_1:
	xor	edx, edx
	div	edi
	mov	r9, qword ptr [rcx + 8]
	mov	r8, qword ptr [rcx + 24]
	test	r8, r8
	je	.LBB0_11
.LBB0_4:
	lea	rdi, [rsp + 32]
	lea	r10, [8*r8]
	xor	edx, edx
	mov	r11, rax
	xor	esi, esi
	jmp	.LBB0_5
	.p2align	4, 0x90
.LBB0_7:
	add	rsi, 1
	add	rdx, 8
	cmp	r10, rdx
	je	.LBB0_11
.LBB0_5:
	cmp	rsi, rax
	jae	.LBB0_8
	test	rdi, rdi
	jne	.LBB0_7
.LBB0_8:
	cmp	r11, r8
	jae	.LBB0_11
	add	r11, 1
	xor	edi, edi
	add	rdx, 8
	cmp	r10, rdx
	jne	.LBB0_5
.LBB0_11:
	mov	rdx, qword ptr [rcx + 16]
	test	rdx, rdx
	je	.LBB0_14
	shl	rdx, 3
	test	rdx, rdx
	je	.LBB0_14
	mov	r8d, 8
	mov	rcx, r9
	call	__rust_dealloc
.LBB0_14:
	nop
	add	rsp, 40
	pop	rdi
	pop	rsi
	ret
	.seh_handlerdata
	.section	.text,"xr",one_only,_ZN10contextual3asm17h7a821eeb655f3e5dE
	.seh_endproc

