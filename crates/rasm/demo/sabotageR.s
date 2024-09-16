
%define FUNTAB_SIZE 40
%define SEXPREC_HEADER_LEN 32

section .rodata
    crashpls_msg db "Something's seriously wrong, crashing by executing UD2 in 3... 2... 1...", 0
    do_set_not_found_msg db "do_set not found in the function table.", 10, 0
    no_eq_sign_msg db "This is R, use <- instead of `=` :D", 0
    equal_sign db "=", 0
    fmt_s db "%s", 0
    fmt_s_nl db "%s", 10, 0

section .data
    real_do_set dq 0
    eq_assign_call_no dq 0

section .text

    extern Rf_errorcall
    extern Rf_error
    extern Rf_ScalarInteger
    extern R_ShowMessage
    extern strcmp
    extern R_FunTab
    extern R_CHAR
    extern Rf_asChar

sabotage: ; SEXP(SEXP) // replace the do_set function pointer in the function table
    call Rf_asChar
    mov rdi, rax
    call R_CHAR
    mov r14, rax
    mov r12, R_FunTab ; lea is too far away :(
    sub r12, FUNTAB_SIZE
    mov r13d, 0
    .loop:
        inc r13d
        add r12, FUNTAB_SIZE
        mov rax, [r12]
        test rax, rax
        jz .notfound
        mov rdi, rax 
        mov rsi, r14 
        call strcmp
        test rax, rax
        jnz .loop

        lea rax, [r12 + 8] ; get the function pointer
        mov rcx, real_do_set
        mov [rcx], rax ; save the original function pointer
        dec r13d
        mov rcx, eq_assign_call_no
        mov [rcx], r13d ; save the index

        mov r13, __patched_do_set
        mov DWORD [r12 + 8], r13d ; patch the table, evil >:)

        mov rdi, 0x1
        jmp Rf_ScalarInteger
    .notfound:
        xor rdi, rdi
        jmp Rf_ScalarInteger

__patched_do_set: ; SEXP(SEXP, SEXP, SEXP, SEXP) // rdi is the call, rsi is the discr
    push rdi
    push rsi
    push rdx
    push rcx

    xor rcx, rcx
    mov ecx, DWORD [rsi + SEXPREC_HEADER_LEN]
    mov r12, eq_assign_call_no ; just a demo, not really complete
    cmp rcx, [r12]
    je .is_equal_sign
    
    pop rcx
    pop rdx
    pop rsi
    pop rdi

    mov r12, real_do_set
    jmp [r12]
    jmp crashpls
    
    .is_equal_sign:
        push rcx
        xor eax, eax
        mov rsi, fmt_s_nl
        mov rdx, no_eq_sign_msg
        call Rf_errorcall
        jmp crashpls

crashpls:
    mov rdi, crashpls_msg
    call R_ShowMessage
    ud2
