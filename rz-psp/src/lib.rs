mod types;
use crate::types::*;

use core::{
    ffi::{c_char, c_void, c_int, CStr},
    ptr, mem,
    convert::TryInto,
};

use std::{ffi::CString, collections::HashMap, cell::RefCell};

use rizin_librz_sys::{
    RzCorePlugin, RzCore, RzLibStruct, RzLibType_RZ_LIB_TYPE_CORE,
    rz_cmd_get_root, rz_cmd_desc_argv_new, 
    rz_cmd_status_t_RZ_CMD_STATUS_WRONG_ARGS, rz_cmd_status_t_RZ_CMD_STATUS_OK,
    rz_cmd_status_t_RZ_CMD_STATUS_INVALID,
    RzCmdDescHelp, RzCmdDescArg, rz_cmd_arg_type_t_RZ_CMD_ARG_TYPE_STRING,
    RzCmdStatus, RzCmdDescDetail, RzCmdDescDetailEntry, rz_io_read_at, RzBinSection,
    rz_bin_object_get_sections_all, rz_type_parse_string,
    rz_analysis_var_global_create, rz_type_db_get_struct, 
    rz_type_identifier_of_base_type, rz_type_array_of_base_type,
    rz_analysis_get_function_at, rz_analysis_function_rename,
    rz_bin_cur_object, RzPVector, rz_pvector_pop_front, rz_analysis_create_function,
    RzAnalysisFcnType_RZ_ANALYSIS_FCN_TYPE_FCN, rz_bin_object_p2v
};

use serde::{Serialize, Deserialize};

#[no_mangle]
static mut RZ_CORE_PLUGIN_PSP: RzCorePlugin = RzCorePlugin {
	name: b"rz-psp\0".as_ptr() as *const c_char,
	desc: b"Sony PSP Reverse Engineering Plugin for Rizin\0".as_ptr() as *const c_char,
	license: b"MIT\0".as_ptr() as *const c_char,
	version: b"0.1.0\0".as_ptr() as *const c_char,
	author: b"sajattack\0".as_ptr() as *const c_char,
	init:  Some(rz_cmd_psp_init),
	fini: None,
	analysis: None,
};


static mut NID_MAP: RefCell<Option<HashMap<u32, &str>>> = RefCell::new(None);

#[no_mangle]
pub unsafe extern "C" fn rz_cmd_psp_init(core: *mut RzCore) -> bool 
{
    //println!("Hello World!");
    if core.is_null() { return false; }
    let rcmd = unsafe {*core}.rcmd;
    let root_cd = unsafe { rz_cmd_get_root(rcmd) };
    if root_cd.is_null() { return false; }
    let cd = unsafe { rz_cmd_desc_argv_new(rcmd, root_cd, b"rz-psp\0".as_ptr() as *const c_char, Some(rz_cmd_psp_handler), &CMD_PSP_HELP) };
    if cd.is_null() { return false; }


    let psp_header = include_str!("../assets/pspsdk_types.h");
    let psp_niddb = include_str!("../assets/niddb_combined.xml");

    let parsed_niddb: FunctionVec = quick_xml::de::from_str(psp_niddb).unwrap();
    //println!("{:?}", parsed_niddb.clone());

    unsafe { NID_MAP = RefCell::new(Some(HashMap::new())); }
    let mut local_borrow = unsafe { NID_MAP.borrow_mut() };
    let local_map = local_borrow.as_mut().unwrap();
    for entry in parsed_niddb.functions.into_iter() {
        local_map.insert(u32::from_str_radix(&entry.nid[2..], 16).unwrap(), entry.name);
    }

    let anal = unsafe { (*core).analysis };
    let typedb = unsafe { (*anal).typedb };

    // load psp types
    let mut error = vec![vec![0u8; 1024]; 10];
    let psp_header_c = CString::new(psp_header).unwrap();
    if unsafe { rz_type_parse_string(typedb, psp_header_c.as_ptr(), error.as_mut_ptr() as *mut *mut _) } != 0 {
        //println!("{}", CStr::from_bytes_until_nul(&error[0]).unwrap().to_str().unwrap());
    }

    true    
}

#[no_mangle]
pub extern "C" fn rz_cmd_psp_handler(core: *mut RzCore, argc: c_int, argv: *mut *const c_char) -> RzCmdStatus {
    if argc != 2 {
        return rz_cmd_status_t_RZ_CMD_STATUS_WRONG_ARGS
    } else {
        if unsafe{ !argv.add(1).is_null() } {
            if let Ok(arg) = unsafe { CStr::from_ptr(*argv.add(1)) }.to_str() {
                if arg == "nid" {
                    do_nid_stuff(core);
                    return  rz_cmd_status_t_RZ_CMD_STATUS_OK
                }
                return rz_cmd_status_t_RZ_CMD_STATUS_INVALID
            }
            return rz_cmd_status_t_RZ_CMD_STATUS_INVALID
        }
        rz_cmd_status_t_RZ_CMD_STATUS_INVALID
    }
}

#[no_mangle]
static mut CMD_PSP_HELP: RzCmdDescHelp = RzCmdDescHelp {
    summary: b"Sony PSP Reverse Engineering Plugin\0".as_ptr() as *const c_char,
    args: unsafe { CMD_PSP_ARGS.as_ptr() },
    description: ptr::null(),
    args_str: ptr::null(),
    usage:  b"rz-psp subcommand\0".as_ptr() as *const c_char,
    options: ptr::null(),
    sort_subcommands: true,
    details: unsafe { CMD_PSP_DETAILS.as_ptr() },
    details_cb: None,
};

#[no_mangle]
static mut CMD_PSP_DETAIL_ENTRY: RzCmdDescDetailEntry = RzCmdDescDetailEntry {
    text: b"nid\0".as_ptr() as *const c_char,
    comment: b"Resolve NIDs to function names\0".as_ptr() as *const c_char,
    arg_str: ptr::null(),
};

#[no_mangle]
static mut CMD_PSP_DETAILS: [RzCmdDescDetail;2] = [
    RzCmdDescDetail {
        name: b"subcommand\0".as_ptr() as *const c_char,
        entries: unsafe { [CMD_PSP_DETAIL_ENTRY, mem::zeroed()].as_ptr() }
    },
    unsafe { mem::zeroed() },
];

#[no_mangle]
static mut CMD_PSP_ARGS: [RzCmdDescArg;2] = [RzCmdDescArg {
    name: b"subcommand\0".as_ptr() as *const c_char,
    type_: rz_cmd_arg_type_t_RZ_CMD_ARG_TYPE_STRING,
    optional: false, 
    no_space: false, 
    flags: 0,
    default_value: ptr::null(),
    __bindgen_anon_1: unsafe { mem::zeroed() },
},
    unsafe { mem::zeroed() }
];

/// A generic converter from RzVector to Vec (with a type conversion function)
macro_rules! rzpvector_to_vec {
    ($fn_name:ident, $return_type:ident, $size:expr) => {
        fn $fn_name(pvec: *mut RzPVector) -> Vec<$return_type> {
            let mut vec: Vec<$return_type> = Vec::new();
            unsafe {
                while (*pvec).v.len != 0 {
                    let element = rz_pvector_pop_front(pvec) as *mut u8;
                    let mut buf = [0u8; $size];
                    element.copy_to(buf.as_mut_ptr(), $size);
                    // FIXME maybe bytemuck this if you can find a decent
                    // way to derive the trait required???
                    let el = core::mem::transmute::<[u8; $size], $return_type>(buf);
                    vec.push(el);
                }
            }
            vec
        }
    };
}

/// A generic converter from RzList to Vec
macro_rules! rzlist_to_vec {
    ($fn_name:ident,  $return_type:ident, $size:expr) => {
        fn $fn_name(rzl: *mut rizin_librz_sys::RzList) -> Vec<$return_type> {
            let mut vec: Vec<$return_type> = Vec::new();
            unsafe {
                let mut ptr = (*rzl).head;
                while !ptr.is_null() {
                    let data = (*ptr).elem;
                    let element = data as *mut u8;
                    let mut buf = [0u8; $size];
                    element.copy_to(buf.as_mut_ptr(), $size);
                    // FIXME maybe bytemuck this if you can find a decent
                    // way to derive the trait required???
                    let el = core::mem::transmute::<[u8; $size], $return_type>(buf);
                    vec.push(el);
                    ptr = (*ptr).next;
                }
            }
            vec
        }
    };
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
struct Function<'a> {
    #[serde(rename="NID")]
    nid: &'a str,
    #[serde(rename="NAME")]
    name: &'a str,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename="FUNCTIONS")]
struct FunctionVec<'a> {
    #[serde(rename="FUNCTION", default)]
    #[serde(borrow)]
    functions: Vec<Function<'a>>,
}

rzpvector_to_vec!(section_list_to_vec, RzBinSection, core::mem::size_of::<RzBinSection>());

fn do_nid_stuff(core: *mut RzCore) {
    //println!("Hello NID");

    let anal = unsafe { (*core).analysis };
    let typedb = unsafe { (*anal).typedb };
    let io = unsafe { (*core).io };
    //unsafe { (*io).va = false as i32; } // use physical addreses
    let bin = unsafe { (*core).bin };
    let bobj = unsafe { rz_bin_cur_object(bin) };

    if bobj.is_null() {
        println!("A file must be loaded first");
        return;
    }


    let sections = unsafe { rz_bin_object_get_sections_all(bobj) as *mut _ };
    let sections = section_list_to_vec(sections);
    for s in sections {
        let name = unsafe { CStr::from_ptr(s.name).to_str().unwrap()};
        if name == ".rodata.sceModuleInfo" {
            let mut buf = Vec::new();
            buf.resize(s.size.try_into().unwrap(), 0);
            unsafe { rz_io_read_at(io, rz_bin_object_p2v(bobj, s.paddr), buf.as_mut_ptr(), s.size.try_into().unwrap()) };
            let modinfo = bytemuck::from_bytes::<PspModuleInfo>(&buf[..core::mem::size_of::<PspModuleInfo>()]);
            let rz_modinfo_type = unsafe { rz_type_identifier_of_base_type(typedb, rz_type_db_get_struct(typedb, b"PspModuleInfo\0".as_ptr() as *const _), false)};
            let rz_modinfo_var = unsafe { rz_analysis_var_global_create(anal, b"var_module_info\0".as_ptr() as *const _, rz_modinfo_type, s.vaddr) };
            
            let mut buf = Vec::new();
            let exports_size = modinfo.exports_end_addr - modinfo.exports_addr;
            let exports_count = exports_size as usize / core::mem::size_of::<PspModuleExport>();
            buf.resize(exports_size as usize, 0);
            let exports_paddr = modinfo.exports_addr as u64 + EHDR_SIZE as u64;
            let exports_vaddr = unsafe { rz_bin_object_p2v(bobj, exports_paddr)};
            unsafe { rz_io_read_at(io, exports_paddr, buf.as_mut_ptr(), exports_size as u64) };
            let export_bytes = buf;
            let exports = bytemuck::allocation::pod_collect_to_vec::<u8, PspModuleExport>(&export_bytes);
            //dbg!(exports.clone());
            let rz_exports_type = unsafe { rz_type_array_of_base_type(typedb, rz_type_db_get_struct(typedb, "PspModuleExport\0".as_ptr() as *const _), exports_count as u64)};
            let rz_exports_var = unsafe { rz_analysis_var_global_create(anal, format!("PspModuleExport_{:08x}\0", exports_vaddr).as_bytes().as_ptr() as *const _, rz_exports_type, exports_vaddr) };

            for exp in exports {
                let exp: PspModuleExport = exp;
                let mut buf = [0u8;PSP_ENTRY_MAX_NAME];
                unsafe { rz_io_read_at(io, rz_bin_object_p2v(bobj, exp.name.into()), buf.as_mut_ptr(), PSP_LIB_MAX_NAME as u64); }
                //let export_name = CStr::from_bytes_until_nul(&buf).unwrap().to_str().unwrap();
                //let rz_exports_entry_type = unsafe { rz_type_array_of_base_type(typedb, rz_type_db_get_struct(typedb, "SceLibraryEntryTable\0".as_ptr() as *const _), exports_count as u64)};
                //let rz_exports_entry_var = unsafe { rz_analysis_var_global_create(anal, b"entry\0".as_ptr() as *const _, rz_exports_entry_type, rz_bin_object_p2v(bobj, exp.exports as u64)) };
            }

            let mut buf = Vec::new();
            let imports_size = modinfo.imports_end_addr - modinfo.imports_addr;
            let imports_count = imports_size as usize / core::mem::size_of::<PspModuleImport>();
            buf.resize(imports_size as usize, 0);
            let imports_paddr = modinfo.imports_addr as u64 + EHDR_SIZE as u64;
            let imports_vaddr = unsafe { rz_bin_object_p2v(bobj, imports_paddr)};
            unsafe { rz_io_read_at(io, imports_vaddr, buf.as_mut_ptr(), imports_size as u64) };
            let import_bytes = buf;
            let imports = bytemuck::allocation::pod_collect_to_vec::<u8, PspModuleImport>(&import_bytes);
            let rz_imports_type = unsafe { rz_type_array_of_base_type(typedb, rz_type_db_get_struct(typedb, "PspModuleImport\0".as_ptr() as *const _), imports_count as u64)};
            let rz_imports_var = unsafe { rz_analysis_var_global_create(anal, format!("PspModuleImport_{:08x}\0", imports_vaddr).as_ptr() as *const _, rz_imports_type, imports_vaddr) };


            if let Some(local_map) = unsafe {NID_MAP.borrow().as_ref()} {
                for imp in imports {
                    let imp: PspModuleImport = imp;
                    let mut buf = [0u8;PSP_LIB_MAX_NAME];
                    unsafe { rz_io_read_at(io, rz_bin_object_p2v(bobj, imp.name.into()), buf.as_mut_ptr(), PSP_LIB_MAX_NAME as u64); }
                    //let _import_name = CStr::from_bytes_until_nul(&buf).unwrap().to_str().unwrap();
                    for i in 0..imp.func_count {
                        let nid_addr: u64 = imp.nid_addr as u64 + 4*i as u64;
                        let mut nid: u32 = 0;
                        unsafe { rz_io_read_at(io, rz_bin_object_p2v(bobj, (nid_addr + EHDR_SIZE as u64)), ptr::addr_of_mut!(nid) as *mut _, 4u64) };
                        //println!("nid: {:08x}", nid);
                        let name = local_map.get(&nid);
                        match name {
                            Some(n) => {
                                let paddr = (imp.funcs_addr+4*i as u32) as u64;
                                let vaddr = unsafe { rz_bin_object_p2v(bobj, paddr) };
                                let name_cstr = CString::new(*n).unwrap();
                                //println!("{} @ {:08x}", n, vaddr);
                                let existing_func = unsafe { rz_analysis_get_function_at(anal, paddr) };
                                if existing_func.is_null()
                                {
                                    let func = unsafe { rz_analysis_create_function(anal, name_cstr.as_ptr(), paddr, RzAnalysisFcnType_RZ_ANALYSIS_FCN_TYPE_FCN) };
                                } else {
                                    unsafe { rz_analysis_function_rename(existing_func, name_cstr.as_ptr()) };
                                }
                            },
                            None => {

                            },
                        }

                    }
                }
            }
            //for exp in exports {
                //dbg!(imp);
                //println!("{:x?}", exp);
            //}
        } else if name == ".rodata.sceNid" {
            let mut buf = Vec::new();
            buf.resize(s.size.try_into().unwrap(), 0);
            unsafe { rz_io_read_at(io, rz_bin_object_p2v(bobj, s.paddr), buf.as_mut_ptr(), s.size.try_into().unwrap()) };
        }
    }
    //unsafe { (*io).va = true as i32; } // use virtual addreses
}

#[no_mangle]
pub static mut rizin_plugin: RzLibStruct = RzLibStruct {
    type_: RzLibType_RZ_LIB_TYPE_CORE,
    data: unsafe { &RZ_CORE_PLUGIN_PSP as *const _ as *mut c_void }, // lolwut?
    version: b"0.7.2\0".as_ptr() as *const _,
    free: None,
};


