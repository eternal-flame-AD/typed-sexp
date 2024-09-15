//! # Rasm
//!
//! Rasm is a simple library that allows you to assemble x86_64 assembly code and call it from R.
#![warn(missing_docs)]
use std::{alloc::Layout, collections::HashMap, ffi::c_int, marker::PhantomPinned};

mod relocate;

use elf::{abi::STV_DEFAULT, endian::AnyEndian, section::SectionHeader, ElfBytes};
use relocate::ComputeRelocation;
use typed_sexp::{
    prelude::*,
    sexp::{ptr::Ptr, vector::List},
};

#[cfg(not(target_arch = "x86_64"))]
compile_error!("Only x86_64 is supported (for now)");

#[cfg(not(target_os = "linux"))]
compile_error!("Only Linux is supported (for now)");

/// Type-level marker for an ISA.
pub unsafe trait ISA {}

/// x86_64 ISA. Currently the only supported ISA.
pub struct X64ISA;

unsafe impl ISA for X64ISA {}

/// A page of memory.
pub struct Page {
    paged: bool,
    ptr: *mut u8,
    size: usize,
}

impl Page {
    /// Create a new page of memory.
    pub fn new(size: usize) -> Self {
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };

        if ptr == libc::MAP_FAILED {
            panic!("Failed to allocate memory");
        }

        Self {
            ptr: ptr.cast(),
            size,
            paged: true,
        }
    }

    /// Change the protection of the page.
    pub fn mprotect(&mut self, prot: c_int) -> Result<(), c_int> {
        unsafe {
            if libc::mprotect(self.ptr.cast(), self.size, prot) != 0 {
                Err(*libc::__errno_location())
            } else {
                Ok(())
            }
        }
    }

    /// Create a non-paged page of memory. This is useful for RW- sections.
    pub fn non_paged(size: usize, align: usize) -> Self {
        unsafe {
            let buf = std::alloc::alloc(Layout::from_size_align(size, align).unwrap());
            Self {
                ptr: buf,
                size,
                paged: false,
            }
        }
    }

    /// Add an offset to the pointer.
    pub fn byte_add(&self, offset: usize) -> *mut u8 {
        self.ptr.wrapping_add(offset)
    }

    /// Get the pointer to the memory.
    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr
    }
}

impl Drop for Page {
    fn drop(&mut self) {
        if self.paged {
            unsafe {
                libc::munmap(self.ptr.cast(), self.size);
            }
        } else {
            unsafe {
                std::alloc::dealloc(self.ptr, Layout::from_size_align(self.size, 1).unwrap());
            }
        }
    }
}

/// An assembled and linked module.
pub struct AsmFunction<I: ISA> {
    text: Page,
    #[allow(unused)]
    data: Option<Page>,
    #[allow(unused)]
    rodata: Option<Page>,
    /// Function offset table.
    func: HashMap<String, usize>,
    _pin: PhantomPinned,
    _isa: std::marker::PhantomData<I>,
}

macro_rules! generate_asmcall {
    ($($name:ident( $( $arg_name:ident: $arg_ty:ident ),*))*) => {
        $(
            /// Call a function by name.
            #[cfg_attr(feature = "clobber_less", inline(never))] // let Rust clean up the registers as this function returns
            #[cfg_attr(not(feature = "clobber_less"), inline)]
            pub unsafe fn $name<R $(, $arg_ty)*>(&self, name: &str $(, $arg_name: $arg_ty)*) -> R {
                    let func =
                        self.text.as_ptr()
                            .byte_add(*self.func.get(name).expect("Function not found"));

                    let func = std::mem::transmute::<*const _, extern "C" fn($($arg_ty),*) -> R>(func);

                    func($($arg_name),*)
            }
        )*
    }
}

impl<I: ISA> AsmFunction<I> {
    /// Create a new module from ELF bytes.
    pub fn new_elf(elf_bytes: &[u8]) -> Self {
        let parsed = ElfBytes::<AnyEndian>::minimal_parse(elf_bytes).unwrap();
        let common_data = parsed.find_common_data().unwrap();

        let data = parsed.section_header_by_name(".data").unwrap();
        let rodata = parsed.section_header_by_name(".rodata").unwrap();

        let symtab = common_data.symtab.as_ref().expect("No symbol table");
        let symtab_str = common_data
            .symtab_strs
            .as_ref()
            .expect("No symbol table strings");

        let text = parsed
            .section_header_by_name(".text")
            .unwrap()
            .expect("No .text section");

        let text_idx = parsed
            .section_headers()
            .unwrap()
            .iter()
            .position(|x| x == text)
            .unwrap();

        let alloc_section_page = |header: SectionHeader, prot: Option<c_int>| {
            let size = header.sh_size as usize;

            match prot {
                Some(prot) => {
                    let mut page = Page::new(size);

                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            elf_bytes.as_ptr().add(header.sh_offset as usize),
                            page.as_ptr(),
                            size,
                        );
                    }

                    page.mprotect(prot).unwrap();

                    page
                }
                None => {
                    let page = Page::non_paged(size, header.sh_addralign as usize);

                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            elf_bytes.as_ptr().add(header.sh_offset as usize),
                            page.as_ptr(),
                            size,
                        );

                        page
                    }
                }
            }
        };

        let mut text_page = alloc_section_page(text, Some(libc::PROT_READ | libc::PROT_WRITE));
        let data_page = data.map(|h| alloc_section_page(h, None));
        let rodata_page = rodata.map(|h| alloc_section_page(h, Some(libc::PROT_READ)));

        let mut func_table = HashMap::new();

        for sym in symtab.iter() {
            if sym.st_vis() != STV_DEFAULT || sym.st_name == 0 || sym.st_shndx != text_idx as _ {
                continue;
            }

            let name = symtab_str
                .get(sym.st_name as _)
                .expect("No symbol table strings");

            if name.is_empty() || name.starts_with("__") {
                continue;
            }

            let name = name.to_string();

            log::info!("Adding symbol: {:?}", name);

            func_table.insert(name, sym.st_value as _);
        }

        ComputeRelocation {
            parsed: &parsed,
            common_data: &common_data,
            data: match &data_page {
                Some(p) => Some((data.as_ref().unwrap(), p)),
                None => None,
            },
            rodata: match &rodata_page {
                Some(p) => Some((rodata.as_ref().unwrap(), p)),
                None => None,
            },
            text: &text,
            text_page: &text_page,
            handle: unsafe { libc::dlopen(std::ptr::null(), libc::RTLD_LAZY) },
        }
        .apply_all_relocations();

        text_page
            .mprotect(libc::PROT_READ | libc::PROT_EXEC)
            .unwrap();

        Self {
            text: text_page,
            func: func_table,
            data: data_page,
            rodata: rodata_page,
            _pin: PhantomPinned,
            _isa: std::marker::PhantomData,
        }
    }

    /// Assemble a string into a module.
    pub fn assemble(input: &str) -> Result<Self, String> {
        let tmp_input = std::env::temp_dir().join("rasm_tmp.asm");
        let tmp_output = std::env::temp_dir().join("rasm_tmp.o");

        std::fs::write(&tmp_input, input).map_err(|e| e.to_string())?;

        let output = std::process::Command::new("nasm")
            .arg("-felf64")
            .arg("-o")
            .arg(&tmp_output)
            .arg(&tmp_input)
            .output()
            .expect("Failed to spawn nasm");

        if !output.status.success() {
            return Err(format!(
                "Failed to assemble: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let elf = std::fs::read(&tmp_output).map_err(|e| e.to_string())?;

        let f = Self::new_elf(&elf);

        Ok(f)
    }

    generate_asmcall!(
        call0()
        call1(p0: P0)
        call2(p0: P0, p1: P1)
        call3(p0: P0, p1: P1, p2: P2)
        call4(p0: P0, p1: P1, p2: P2, p3: P3)
        call5(p0: P0, p1: P1, p2: P2, p3: P3, p4: P4)
        call6(p0: P0, p1: P1, p2: P2, p3: P3, p4: P4, p5: P5)
        call7(p0: P0, p1: P1, p2: P2, p3: P3, p4: P4, p5: P5, p6: P6)
        call8(p0: P0, p1: P1, p2: P2, p3: P3, p4: P4, p5: P5, p6: P6, p7: P7)
    );

    #[cfg(feature = "16_args")]
    generate_asmcall!(
        call9(p0: P0, p1: P1, p2: P2, p3: P3, p4: P4, p5: P5, p6: P6, p7: P7, p8: P8)
        call10(p0: P0, p1: P1, p2: P2, p3: P3, p4: P4, p5: P5, p6: P6, p7: P7, p8: P8, p9: P9)
        call11(p0: P0, p1: P1, p2: P2, p3: P3, p4: P4, p5: P5, p6: P6, p7: P7, p8: P8, p9: P9, p10: P10)
        call12(p0: P0, p1: P1, p2: P2, p3: P3, p4: P4, p5: P5, p6: P6, p7: P7, p8: P8, p9: P9, p10: P10, p11: P11)
        call13(p0: P0, p1: P1, p2: P2, p3: P3, p4: P4, p5: P5, p6: P6, p7: P7, p8: P8, p9: P9, p10: P10, p11: P11, p12: P12)
        call14(p0: P0, p1: P1, p2: P2, p3: P3, p4: P4, p5: P5, p6: P6, p7: P7, p8: P8, p9: P9, p10: P10, p11: P11, p12: P12, p13: P13)
        call15(p0: P0, p1: P1, p2: P2, p3: P3, p4: P4, p5: P5, p6: P6, p7: P7, p8: P8, p9: P9, p10: P10, p11: P11, p12: P12, p13: P13, p14: P14)
        call16(p0: P0, p1: P1, p2: P2, p3: P3, p4: P4, p5: P5, p6: P6, p7: P7, p8: P8, p9: P9, p10: P10, p11: P11, p12: P12, p13: P13, p14: P14, p15: P15)
    );
}

mod ffi {
    use super::*;

    // TODO: R registration

    #[export_name = "assemble"]
    /// R external function to assemble a string into a module.
    pub extern "C" fn assemble(input: SEXP) -> SEXP {
        let input = input
            .downcast_to::<CharacterVectorSEXP<_>>()
            .expect_r("input is not a string")
            .protect();

        if input.len() != 1 {
            Err::<(), _>("Expected a single string").unwrap_r();
        }
        let f = Box::new(
            AsmFunction::<X64ISA>::assemble(&input.get_elt(0).to_string())
                .expect_r("Failed to assemble"),
        );

        let ptr_inner = CharacterVectorSEXP::scalar("<asm_function>").protect();

        let ptr = Ptr::<SEXP, AsmFunction<X64ISA>>::wrap_boxed(f, r_nil(), ptr_inner);

        ptr.get_sexp()
    }

    #[export_name = "init_rustlog"]
    /// R external function to initialize the logger.
    pub extern "C" fn init_rustlog() -> SEXP {
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "warn");
        }
        env_logger::init();

        r_nil()
    }

    #[export_name = "asm_call"]
    /// R external function to call a function in an assembled module.
    pub extern "C" fn call(f: SEXP, name: SEXP, param: SEXP) -> SEXP {
        let f = f
            .downcast_to::<Ptr<SEXP, AsmFunction<X64ISA>>>()
            .expect_r("f is not a vector")
            .protect();

        let f = f.get_ref();
        log::debug!("Function pointer: {:p}", f);

        let name = name
            .downcast_to::<CharacterVectorSEXP<_>>()
            .expect_r("name is not a string")
            .protect();

        if name.len() != 1 {
            Err::<(), _>("Expected a single string").unwrap_r();
        }

        let name = name.get_elt(0).to_string();

        let name = name.as_str();

        let param = param
            .downcast_to::<List<_>>()
            .expect_r("param is not a list")
            .protect();

        log::debug!(
            "Dispatching call to: {:?} with {:?} arguments",
            name,
            param.len()
        );

        macro_rules! generate_dispatch {
        ($( $len:literal => $dispatch_fn:ident ($([$idx:literal]),*) ),*) => {
            match param.len() {
                $(
                    $len => {
                        Some(unsafe {
                            f.$dispatch_fn(name $(, param.get_elt($idx))*)
                        })
                    }
                )*
                _ => None
            }
        };
    }

        #[allow(unused_mut)]
        let mut ret = generate_dispatch!(
            0 => call0(),
            1 => call1([0]),
            2 => call2([0], [1]),
            3 => call3([0], [1], [2]),
            4 => call4([0], [1], [2], [3]),
            5 => call5([0], [1], [2], [3], [4]),
            6 => call6([0], [1], [2], [3], [4], [5]),
            7 => call7([0], [1], [2], [3], [4], [5], [6]),
            8 => call8([0], [1], [2], [3], [4], [5], [6], [7])
        );

        #[cfg(feature = "16_args")]
        {
            ret = ret.or_else(|| generate_dispatch!(
        9 => call9([0], [1], [2], [3], [4], [5], [6], [7], [8]),
        10 => call10([0], [1], [2], [3], [4], [5], [6], [7], [8], [9]),
        11 => call11([0], [1], [2], [3], [4], [5], [6], [7], [8], [9], [10]),
        12 => call12([0], [1], [2], [3], [4], [5], [6], [7], [8], [9], [10], [11]),
        13 => call13([0], [1], [2], [3], [4], [5], [6], [7], [8], [9], [10], [11], [12]),
        14 => call14([0], [1], [2], [3], [4], [5], [6], [7], [8], [9], [10], [11], [12], [13]),
        15 => call15([0], [1], [2], [3], [4], [5], [6], [7], [8], [9], [10], [11], [12], [13], [14]),
        16 => call16([0], [1], [2], [3], [4], [5], [6], [7], [8], [9], [10], [11], [12], [13], [14], [15])
    ));
        }

        let ret = ret.expect_r("Unsupported number of arguments");

        log::debug!("Return value: {:p}", ret);

        ret
    }
}
