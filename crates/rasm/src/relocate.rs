use std::ffi::CString;

use elf::{
    abi::{R_X86_64_32, R_X86_64_32S, R_X86_64_64, R_X86_64_PC32, R_X86_64_PLT32},
    endian::AnyEndian,
    relocation::{Rel, Rela},
    section::SectionHeader,
    CommonElfData, ElfBytes,
};

use crate::Page;

pub(crate) struct ComputeRelocation<'a> {
    pub parsed: &'a ElfBytes<'a, AnyEndian>,
    pub common_data: &'a CommonElfData<'a, AnyEndian>,
    pub data: Option<(&'a SectionHeader, &'a Page)>,
    pub rodata: Option<(&'a SectionHeader, &'a Page)>,
    pub text: &'a SectionHeader,
    pub text_page: &'a Page,
    pub handle: *mut libc::c_void,
}

pub trait Applicator {
    fn apply_on_page(&self, dest: &Page);
}

pub struct X6432Applicator {
    pub r_offset: u64,
    pub value: u32,
}

fn apply_addend(input: usize, addend: i64) -> usize {
    if addend < 0 {
        input.wrapping_sub(-addend as usize)
    } else {
        input.wrapping_add(addend as usize)
    }
}

impl X6432Applicator {
    fn new(r_offset: u64, sym_val: usize, addend: i64, sign_extend: bool) -> Self {
        let target_value = apply_addend(sym_val, addend);
        let sign32 = target_value & 0x8000_0000;
        let sign_extend_mask: usize = if sign_extend && sign32 != 0 {
            !0 >> 31 << 31
        } else {
            0
        };
        // We don't really have a say on where our page is mapped, so pointers can be really far
        assert_eq!(
            target_value,
            (target_value & 0xffff_ffff) | sign_extend_mask,
            "Relocation overflow, solution: use 64-bit instructions when accessing relocated data"
        );
        Self {
            r_offset,
            value: target_value as u32,
        }
    }
}

impl Applicator for X6432Applicator {
    fn apply_on_page(&self, dest: &Page) {
        unsafe {
            dest.as_ptr()
                .cast::<u32>()
                .byte_add(self.r_offset as usize)
                .write_unaligned(self.value);
        }
    }
}

pub struct X64PC32Applicator {
    pub r_offset: u64,
    pub value: i64,
}

impl X64PC32Applicator {
    fn new(r_offset: u64, sym_val: usize, addend: i64) -> Self {
        let value = apply_addend(sym_val, addend) as i64;
        Self { r_offset, value }
    }
}

impl Applicator for X64PC32Applicator {
    fn apply_on_page(&self, dest: &Page) {
        // %rip at the relocation point
        let pc = dest.as_ptr() as i64 + self.r_offset as i64;
        unsafe {
            dest.as_ptr()
                .cast::<u32>()
                .byte_add(self.r_offset as usize)
                .write_unaligned(self.value.wrapping_sub(pc) as u32);
        }
    }
}

pub struct X6464Applicator {
    pub r_offset: u64,
    pub value: u64,
}

impl X6464Applicator {
    fn new(r_offset: u64, sym_val: usize, addend: i64) -> Self {
        let value = apply_addend(sym_val, addend) as u64;
        Self { r_offset, value }
    }
}

impl Applicator for X6464Applicator {
    fn apply_on_page(&self, dest: &Page) {
        unsafe {
            dest.as_ptr()
                .cast::<u64>()
                .byte_add(self.r_offset as usize)
                .write_unaligned(self.value);
        }
    }
}

#[derive(Debug, Clone)]
struct RelocRecord {
    r_offset: u64,
    r_addend: i64,
    r_sym: u32,
    r_type: u32,
}

impl From<Rela> for RelocRecord {
    fn from(reloc: Rela) -> Self {
        Self {
            r_offset: reloc.r_offset,
            r_addend: reloc.r_addend,
            r_sym: reloc.r_sym,
            r_type: reloc.r_type,
        }
    }
}

impl From<Rel> for RelocRecord {
    fn from(reloc: Rel) -> Self {
        Self {
            r_offset: reloc.r_offset,
            r_addend: 0,
            r_sym: reloc.r_sym,
            r_type: reloc.r_type,
        }
    }
}

impl<'a> ComputeRelocation<'a> {
    pub fn apply_all_relocations(&'a self) {
        [".rela.text", ".rel.text"]
            .into_iter()
            .flat_map(move |name| {
                self.parsed
                    .section_header_by_name(name)
                    .unwrap()
                    .into_iter()
                    .flat_map(move |hdr| match name {
                        ".rela.text" => self
                            .parsed
                            .section_data_as_relas(&hdr)
                            .expect("rela.text section data is bad")
                            .map(Into::into)
                            .collect::<Vec<_>>(),
                        ".rel.text" => self
                            .parsed
                            .section_data_as_rels(&hdr)
                            .expect("rel.text section data is bad")
                            .map(Into::into)
                            .collect::<Vec<_>>(),
                        _ => unreachable!(),
                    })
            })
            .map(move |rec: RelocRecord| {
                log::debug!("Applying relocation {:?}", rec);
                match rec.r_type {
                    R_X86_64_64 => Box::new(X6464Applicator::new(
                        rec.r_offset,
                        self.resolve_one_symbol(rec.r_sym as _) as _,
                        rec.r_addend,
                    )) as Box<dyn Applicator>,
                    R_X86_64_PC32 | R_X86_64_PLT32 => Box::new(X64PC32Applicator::new(
                        rec.r_offset,
                        self.resolve_one_symbol(rec.r_sym as _) as _,
                        rec.r_addend,
                    )) as Box<dyn Applicator>,
                    R_X86_64_32 | R_X86_64_32S => Box::new(X6432Applicator::new(
                        rec.r_offset,
                        self.resolve_one_symbol(rec.r_sym as _) as _,
                        rec.r_addend,
                        rec.r_type == R_X86_64_32S,
                    )) as Box<dyn Applicator>,
                    _ => {
                        panic!("Unsupported relocation type: {:?}", rec.r_type);
                    }
                }
            })
            .for_each(|applicator| applicator.apply_on_page(self.text_page));
    }

    /// Return the memory address of the pointee of the symbol with the given index.
    pub fn resolve_one_symbol(&self, sym_idx: u32) -> *mut u8 {
        let symtab = self.common_data.symtab.as_ref().expect("No symbol table");
        let symtab_str = self
            .common_data
            .symtab_strs
            .as_ref()
            .expect("No symbol table strings");

        let target_sym = symtab.get(sym_idx as _).expect("No symbol table entry");

        let target_sym_section_idx = target_sym.st_shndx;
        let target_sym_section_hdr = self
            .parsed
            .section_headers()
            .unwrap()
            .get(target_sym_section_idx as usize)
            .expect("No section header");

        let sym_name_idx = target_sym.st_name;

        let sym_name = symtab_str
            .get_raw(sym_name_idx as _)
            .expect("No symbol table strings");

        // first find the symbol in the symbol table
        let sym_val = if !target_sym.is_undefined() {
            let offset = target_sym.st_value as usize;

            let target_is_data = self
                .data
                .as_ref()
                .map_or(false, |(h, _)| **h == target_sym_section_hdr);
            let target_is_rodata = self
                .rodata
                .as_ref()
                .map_or(false, |(h, _)| **h == target_sym_section_hdr);
            let target_is_text = *self.text == target_sym_section_hdr;

            let location = match () {
                _ if target_is_data => self.data.as_ref().unwrap().1.byte_add(offset),
                _ if target_is_rodata => self.rodata.as_ref().unwrap().1.byte_add(offset),
                _ if target_is_text => unsafe { self.text_page.as_ptr().byte_add(offset) },
                _ => panic!("Symbol relocation in unsupported section"),
            };

            log::debug!(
                "Resolved symbol {} to address {:p} (offset {}), is_data: {}, is_rodata: {}, is_text: {}, st_value: {}",
                std::str::from_utf8(sym_name).unwrap(),
                location,
                offset,
                target_is_data,
                target_is_rodata,
                target_is_text,
                target_sym.st_value
            );

            location
        } else {
            let sym = CString::new(sym_name).unwrap();

            let t = unsafe { libc::dlsym(self.handle, sym.as_ptr()) };

            log::debug!(
                "Resolved symbol {} to address {:p} (dlsym)",
                std::str::from_utf8(sym_name).unwrap(),
                t
            );

            if t.is_null() {
                panic!("Symbol not found: {:?}", sym);
            }

            t.cast()
        };

        sym_val
    }
}
