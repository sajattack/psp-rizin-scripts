use core::ffi::c_char;

use bytemuck::{Zeroable, Pod};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum PspEntryType {
	PSP_ENTRY_FUNC = 0,
	PSP_ENTRY_VAR = 1
}

pub const PSP_MODULE_MAX_NAME: usize = 28;
pub const PSP_LIB_MAX_NAME: usize = 128;
pub const PSP_ENTRY_MAX_NAME: usize = 128;
/* the maximum number of permitted entries per lib */
pub const PSP_MAX_V_ENTRIES: usize = 255;
pub const PSP_MAX_F_ENTRIES: usize = 65535;

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct PspModuleInfo {
    pub flags: u32,
    pub name: [c_char; PSP_MODULE_MAX_NAME],
    pub gp_addr: u32,
    pub exports_addr: u32,
    pub exports_end_addr: u32,
    pub imports_addr: u32,
    pub imports_end_addr: u32
}

#[derive(Debug, Clone, Copy, Default, Zeroable, Pod)]
#[repr(C)]
pub struct PspModuleImport {
    pub name: u32,
    pub flags: u32,
    pub entry_size: u8,
    pub var_count: u8,
    pub func_count: u16,
    pub nid_addr: u32,
    pub funcs_addr: u32,
}

#[derive(Debug, Clone, Copy, Default, Zeroable, Pod)]
#[repr(C)]
pub struct PspModuleExport {
    pub name: u32,
    pub flags: u32,
    pub counts: u32,
    pub exports: u32
}

//#[derive(Debug, Clone, Copy, Default, Pod)]
//#[repr(C)]
//struct PspLibImport {
    //prev: *mut PspLibImport,
    //next: *mut PspLibImport,
    //name: [c_char; PSP_LIB_MAX_NAME],
    //vaddr: u32,
    //str
//}

//#[derive(Debug, Clone, Copy, Default, Pod)]
//#[repr(C)]
//struct PspLibExport {
//}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PspEntry {
    pub name: [c_char; PSP_ENTRY_MAX_NAME],
    pub nid: u32,
    pub entry_type: PspEntryType,
    pub vaddr: u32,
    pub nid_vaddr: u32,
}
