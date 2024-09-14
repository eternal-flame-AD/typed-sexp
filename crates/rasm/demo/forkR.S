section .rodata
    notice_msg db "Forking a child process...", 0
    success_child_msg db "I'm the child process from ASM!", 0
    success_parent_msg db "I'm the parent process from ASM!", 0
    success_waitpid_fmt db "Child process exited with status %d", 10, 0
    err_fmt db "Fork failed: %ld", 10, 0
    err_waitpid_fmt db "Waitpid failed: %ld", 10, 0

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

section .text
extern Rf_ScalarInteger
extern Rf_protect
extern Rf_unprotect
extern R_ShowMessage
extern Rf_error
extern Rprintf
extern INTEGER

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

crashpls:
    ud2
