section .rodata
    notice_msg db "Forking a child process...", 0
    success_child_msg db "I'm the child process from ASM!", 0
    success_parent_msg db "I'm the parent process from ASM!", 0
    success_waitpid_fmt db "Child process exited with status %d", 10, 0
    err_fmt db "Fork failed: %ld", 10, 0
    err_waitpid_fmt db "Waitpid failed: %ld", 10, 0
    crashpls_msg db "Crashing by executing UD2 in 3... 2... 1...", 0
    do_set_not_found_msg db "do_set not found in the function table.", 10, 0
    no_eq_sign_msg db "This is R, use <- instead of `=` :D", 0
    equal_sign db "=", 0
    found_injection_msg db "Found the injection point at %p", 10, 0
    fmt_s db "%s", 0
    fmt_s_nl db "%s", 10, 0

section .data
    counter dq 0
    real_do_set dq 0
    eq_assign_call_no dq 0

%define P_PID 1
%define WEXITED 4
%define sys_exit 60
%define sys_fork 57
%define sys_waitid 247
%define SIGINFO_T_SIZE 128
%define SIGINFO_T_SIGNO_OFFSET 0
%define SIGINFO_T_ERRNO_OFFSET 4
%define SIGINFO_T_CODE_OFFSET 8
%define SIGINFO_T_TRAPNO_OFFSET 12
%define SIGINFO_T_PID_OFFSET 16
%define SIGINFO_T_UID_OFFSET 20
%define SIGINFO_T_STATUS_OFFSET 24
%define FUNTAB_SIZE 40
%define SEXPREC_HEADER_LEN 32

section .text
extern Rf_ScalarInteger
extern Rf_protect
extern Rf_unprotect
extern R_ShowMessage
extern Rf_error
extern Rf_errorcall
extern Rprintf
extern INTEGER
extern R_FunTab
extern strcmp

forkr: ; SEXP(void)
    mov rdi, notice_msg
    call R_ShowMessage
    xor eax, eax

    ; int pid = fork();
    mov rax, sys_fork
    syscall
    test rax, rax
    js .error
    push rax
    
    mov edi, eax ; this technically truncates, but I don't care this is just for fun
    call Rf_ScalarInteger
    mov rdi, rax
    call Rf_protect

    push rax

    mov rdi, success_parent_msg
    mov r8, success_child_msg
    mov rcx, [rsp + 8]
    test rcx, rcx
    cmovz rdi, r8
    mov rsi, rax
    call R_ShowMessage
    xor eax, eax

    mov rdi, 0x1
    call Rf_unprotect

    pop rax
    pop rcx
    ret

.error:
    mov rdi, err_fmt
    mov rsi, rax
    call Rf_error
    ud2

exitr:; !(SEXP)
    call INTEGER
    mov rdi, [rax]
    mov rax, sys_exit
    syscall
    ud2

waitpidr: ; SEXP(SEXP)
    call INTEGER; now %rax is the pointer to the pid
    mov r12, [rax]; save for later
    mov rax, 0

    ; set up for waitid
    ; %rdi = which
    mov rdi, P_PID
    ; %rsi = pid
    mov rsi, r12
    ; %rdx = siginfo_t
    sub rsp, SIGINFO_T_SIZE
    mov rdx, rsp
    ; %r10 = options
    mov r10, WEXITED
    ; reserved
    xor r8, r8

    mov rax, sys_waitid
    syscall
    test rax, rax
    js .error

    xor rdi, rdi
    mov edi, DWORD [rsp + SIGINFO_T_CODE_OFFSET]
    add rsp, SIGINFO_T_SIZE
    jmp Rf_ScalarInteger

.error:
    mov rdi, err_waitpid_fmt
    mov rsi, rax
    call Rf_error
    ud2

atomic_fetch_add_u64: ; SEXP(SEXP)
    call INTEGER
    mov r12, [rax]
    mov rax, counter
    xor rdi, rdi
    lock xadd [rax], r12
    mov rdi, r12
    jmp Rf_ScalarInteger

sabotage: ; SEXP() // replace the do_set function pointer in the function table
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
        mov rsi, equal_sign 
        call strcmp; strcmp(rax, "=")
        test rax, rax
        jnz .loop

        lea rax, [r12 + 8] ; get the function pointer
        mov rcx, real_do_set
        mov [rcx], rax ; save the original function pointer
        dec r13d
        mov rcx, eq_assign_call_no
        mov [rcx], r13d ; save the index

        mov r13, __patched_do_set
        mov DWORD [r12 + 8], r13d

        xor rdi, rdi
        jmp Rf_ScalarInteger
    .notfound:
        push rcx
        mov rdi, do_set_not_found_msg
        xor rsi, rsi
        xor rax, rax
        call Rf_error
        ud2

__patched_do_set: ; SEXP(SEXP, SEXP, SEXP, SEXP) // rdi is the call, rsi is the discr
    push rdi
    push rsi
    push rdx
    push rcx

    xor rcx, rcx
    mov ecx, DWORD [rsi + SEXPREC_HEADER_LEN]
    mov r12, eq_assign_call_no
    cmp rcx, [r12]
    je .is_equal_sign
    
    pop rcx
    pop rdx
    pop rsi
    pop rdi

    mov r12, real_do_set
    jmp [r12]
    ud2
    
    .is_equal_sign:
        push rcx
        xor eax, eax
        mov rsi, fmt_s_nl
        mov rdx, no_eq_sign_msg
        call Rf_errorcall
        ud2

crashpls:
    mov rdi, crashpls_msg
    call R_ShowMessage
    ud2
