mod types;
use crate::types::*;

use core::{
    ffi::{c_char, c_void, c_int, CStr, c_ulonglong},
    ptr, mem,
    convert::TryInto,
};

use rizin_librz_sys::{
    RzCorePlugin, RzCore, rz_core_plugin_t, RzLibStruct, RzLibType_RZ_LIB_TYPE_CORE,
    RzCmd, RzCmdDesc, rz_cmd_get_root, rz_cmd_desc_argv_new, 
    rz_cmd_status_t_RZ_CMD_STATUS_WRONG_ARGS, rz_cmd_status_t_RZ_CMD_STATUS_OK,
    rz_cmd_status_t_RZ_CMD_STATUS_INVALID,
    RzCmdDescHelp, RzCmdDescArg, rz_cmd_arg_type_t_RZ_CMD_ARG_TYPE_STRING,
    RzCmdStatus, RzCmdDescDetail, RzCmdDescDetailEntry,
    rz_io_read_at, rz_io_read_at_mapped, rz_io_size, RzBinSection, rz_bin_get_sections,
    rz_io_p2v, rz_bin_object_get_string_at,
};

use bytemuck::from_bytes; 

const rz_core_plugin_psp: RzCorePlugin = RzCorePlugin {
	name: b"rz-psp\0".as_ptr() as *const c_char,
	desc: b"Sony PSP Reverse Engineering Plugin for Rizin\0".as_ptr() as *const c_char,
	license: b"MIT\0".as_ptr() as *const c_char,
	version: b"0.1.0\0".as_ptr() as *const c_char,
	author: b"sajattack\0".as_ptr() as *const c_char,
	init:  Some(rz_cmd_psp_init),
	fini: None,
	analysis: None,
};

#[no_mangle]
pub unsafe extern "C" fn rz_cmd_psp_init(core: *mut RzCore) -> bool 
{
    println!("Hello World!");
    if core.is_null() { return false; }
    let rcmd = unsafe {*core}.rcmd;
    let root_cd = unsafe { rz_cmd_get_root(rcmd) };
    if root_cd.is_null() { return false; }
    let cd = unsafe { rz_cmd_desc_argv_new(rcmd, root_cd, b"rz-psp\0".as_ptr() as *const c_char, Some(rz_cmd_psp_handler), &cmd_psp_help) };
    if cd.is_null() { return false; }
    true    
}

#[no_mangle]
pub unsafe extern "C" fn rz_cmd_psp_handler(core: *mut RzCore, argc: c_int, argv: *mut *const c_char) -> RzCmdStatus {
    if argc != 2 {
        return rz_cmd_status_t_RZ_CMD_STATUS_WRONG_ARGS
    } else {
        if  CStr::from_ptr(*argv.offset(1)).to_str().unwrap() == "nid" {
            do_nid_stuff(core);
            rz_cmd_status_t_RZ_CMD_STATUS_OK
        } else {
            rz_cmd_status_t_RZ_CMD_STATUS_INVALID
        }
    }
}

static mut cmd_psp_help: RzCmdDescHelp = RzCmdDescHelp {
    summary: b"Sony PSP Reverse Engineering Plugin\0".as_ptr() as *const c_char,
    args: unsafe { cmd_psp_args.as_ptr() },
    description: ptr::null(),
    args_str: ptr::null(),
    usage:  b"rz-psp subcommand\0".as_ptr() as *const c_char,
    options: ptr::null(),
    sort_subcommands: true,
    details: [RzCmdDescDetail {
        name: b"subcommand\0".as_ptr() as *const c_char,
        entries: [
            RzCmdDescDetailEntry {
                text: b"nid\0".as_ptr() as *const c_char,
                comment: b"Resolve NIDs to function names\0".as_ptr() as *const c_char,
                arg_str: ptr::null(),
            },
            unsafe { core::mem::zeroed()}
        ].as_ptr()
    }, unsafe { core::mem::zeroed()}].as_ptr() as *const RzCmdDescDetail,
    details_cb: None,
};

static mut cmd_psp_args: [RzCmdDescArg;2] = 
[
    RzCmdDescArg {
        name: b"subcommand\0".as_ptr() as *const c_char,
        type_: rz_cmd_arg_type_t_RZ_CMD_ARG_TYPE_STRING,
        optional: false, 
        no_space: false, 
        flags: 0,
        default_value: ptr::null(),
        __bindgen_anon_1: unsafe { mem::zeroed() },
    },
    unsafe { core::mem::zeroed() }
];

/// A generic converter from RzList to Vec
macro_rules! rzlist_to_vec {
    ($fn_name:ident,  $return_type:ident, $size:expr) => {
        fn $fn_name(rzl: *mut rizin_librz_sys::RzList) -> Vec<$return_type> {
            let mut vec: Vec<$return_type> = Vec::new();
            unsafe {
                let mut ptr = (*rzl).head;
                while !ptr.is_null() {
                    let data = (*ptr).data;
                    let element = data as *mut u8;
                    let mut buf = [0u8; $size];
                    element.copy_to(buf.as_mut_ptr(), $size);
                    // FIXME maybe bytemuck this if you can find a decent
                    // way to derive the trait required???
                    let el = core::mem::transmute::<[u8; $size], $return_type>(buf);
                    vec.push(el);
                    ptr = (*ptr).n;
                }
            }
            vec
        }
    };
}

rzlist_to_vec!(section_list_to_vec, RzBinSection, core::mem::size_of::<RzBinSection>());

fn do_nid_stuff(core: *mut RzCore) {
    println!("Hello NID");
    let io = unsafe { (*core).io };
    unsafe { (*io).va = false as i32; } // use physical addreses
    let bin = unsafe { (*core).bin };

    let bobj = unsafe { (*(*bin).cur).o };

    let sections = unsafe { rz_bin_get_sections(bin) };
    let sections = section_list_to_vec(sections);
    for s in sections {
        let name = unsafe { CStr::from_ptr(s.name).to_str().unwrap()};
        if name == ".rodata.sceModuleInfo" {
            let mut buf = Vec::new();
            buf.resize(s.size.try_into().unwrap(), 0);
            unsafe { rz_io_read_at(io, s.paddr, buf.as_mut_ptr(), s.size.try_into().unwrap()) };
            let modinfo = bytemuck::from_bytes::<PspModuleInfo>(&buf[..core::mem::size_of::<PspModuleInfo>()]);

            let mut buf = Vec::new();
            let exports_size = modinfo.exports_end_addr - modinfo.exports_addr;
            let exports_count = exports_size as usize / core::mem::size_of::<PspModuleExport>();
            buf.resize(exports_size as usize, 0);
            let exports_paddr: u64 = modinfo.exports_addr.try_into().unwrap();
            unsafe { rz_io_read_at(io, exports_paddr, buf.as_mut_ptr(), exports_size as i32) };
            let export_bytes = buf;
            let exports = bytemuck::allocation::pod_collect_to_vec::<u8, PspModuleExport>(&export_bytes);

            let mut buf = Vec::new();
            let imports_size = modinfo.imports_end_addr - modinfo.imports_addr;
            let imports_count = imports_size as usize / core::mem::size_of::<PspModuleImport>();
            buf.resize(imports_size as usize, 0);
            let imports_paddr: u64 = modinfo.imports_addr.try_into().unwrap();
            unsafe { rz_io_read_at(io, imports_paddr, buf.as_mut_ptr(), imports_size as i32) };
            let import_bytes = buf;
            let imports = bytemuck::allocation::pod_collect_to_vec::<u8, PspModuleImport>(&import_bytes);

            for imp in imports {
                unsafe { 
                    let imp: PspModuleImport = imp;
                    let bs = rz_bin_object_get_string_at(bobj, imp.name as c_ulonglong, false);
                    if bs.is_null() {
                        println!("NULL")
                    }
                    else
                    {
                        println!("{}", CStr::from_ptr((*bs).string).to_str().unwrap());
                    }
                }
            }

        } else if name == ".rodata.sceNid" {
            let mut buf = Vec::new();
            buf.resize(s.size.try_into().unwrap(), 0);
            unsafe { rz_io_read_at(io, s.paddr, buf.as_mut_ptr(), s.size.try_into().unwrap()) };
        }
    }
}

#[no_mangle]
pub static mut rizin_plugin: RzLibStruct = RzLibStruct {
    type_: RzLibType_RZ_LIB_TYPE_CORE,
    data: &rz_core_plugin_psp as *const _ as *mut c_void, // lolwut?
    version: b"0.6.3\0".as_ptr() as *const _,
    free: None,
};


