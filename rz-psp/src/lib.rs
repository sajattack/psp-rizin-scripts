use core::{
    ffi::{c_char, c_void, c_int, CStr},
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
    rz_io_read_at, rz_io_size
};


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

static mut cmd_psp_args: [RzCmdDescArg;1] = 
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
];

fn do_nid_stuff(core: *mut RzCore) {
    println!("Hello NID");
    let io = unsafe { (*core).io };
    let mut buf = Vec::new();
    let size = unsafe { rz_io_size(io) };
    buf.resize(size.try_into().unwrap(), 0);
    unsafe { rz_io_read_at(io, 0, buf.as_mut_ptr(), size.try_into().unwrap()); }
    println!("{}", std::string::String::from_utf8(buf.to_vec()).unwrap());
}

#[no_mangle]
pub static mut rizin_plugin: RzLibStruct = RzLibStruct {
    type_: RzLibType_RZ_LIB_TYPE_CORE,
    data: &rz_core_plugin_psp as *const _ as *mut c_void, // lolwut?
    version: b"0.6.3\0".as_ptr() as *const _,
    free: None,
};
