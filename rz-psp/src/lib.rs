use core::{
    ffi::{c_char, c_void, c_int},
    ptr, mem
};

use rizin_librz_sys::{
    RzCorePlugin, RzCore, rz_core_plugin_t, RzLibStruct, RzLibType_RZ_LIB_TYPE_CORE,
    RzCmd, RzCmdDesc, rz_cmd_get_root, rz_cmd_desc_argv_new, 
    rz_cmd_status_t_RZ_CMD_STATUS_WRONG_ARGS, rz_cmd_status_t_RZ_CMD_STATUS_OK,
    RzCmdDescHelp, RzCmdDescArg, rz_cmd_arg_type_t_RZ_CMD_ARG_TYPE_STRING,
    RzCmdStatus, RzCmdDescDetail
};


const rz_core_plugin_psp: RzCorePlugin = RzCorePlugin {
	name: b"rz-psp\0".as_ptr() as *const c_char,
	desc: b"Sony PSP Reverse Engineering Plugin for Rizin\0".as_ptr() as *const c_char,
	license: b"MIT\0".as_ptr() as *const c_char,
	version: b"0.1.0\0".as_ptr() as *const c_char,
	author: b"sajattack\0".as_ptr() as *const c_char,
	init:  Some(rz_cmd_pspg_init),
	fini: None,
	analysis: None,
};

#[no_mangle]
pub unsafe extern "C" fn rz_cmd_pspg_init(core: *mut RzCore) -> bool 
{
    println!("Hello World!");
    let rcmd = (unsafe {*core}).rcmd;
    let root_cd = unsafe { rz_cmd_get_root(rcmd) };
    if root_cd.is_null() {
        return false;
    }
    let cd = unsafe { rz_cmd_desc_argv_new(rcmd, root_cd, b"psp\0".as_ptr() as *const c_char, Some(rz_cmd_psp_handler), &cmd_psp_help) };
    if cd.is_null() {
        //rz_warn_if_reached();
        return false;
    }
    true    
}

#[no_mangle]
pub unsafe extern "C" fn rz_cmd_psp_handler(core: *mut RzCore, argc: c_int, argv: *mut *const c_char) -> RzCmdStatus {
    if argc != 2 {
        rz_cmd_status_t_RZ_CMD_STATUS_WRONG_ARGS
    } else {
        /*if unsafe { libc::strcmp(*argv.offset(1), b"nid\0".as_ptr() as *const c_char) != 0} {
            do_nid_stuff();
            rz_cmd_status_t_RZ_CMD_STATUS_OK
        } else {
            rz_cmd_status_t_RZ_CMD_STATUS_WRONG_ARGS
        }*/
        rz_cmd_status_t_RZ_CMD_STATUS_OK
    }
}

static mut cmd_psp_help: RzCmdDescHelp = RzCmdDescHelp {
    summary: b"Sony PSP Reverse Engineering Plugin\0".as_ptr() as *const c_char,
    args: cmd_psp_args.as_ptr(),
    description: b"\0".as_ptr() as *const c_char,
    args_str: b"\0".as_ptr() as *const c_char,
    usage: b"\0".as_ptr() as *const c_char,
    options: b"\0".as_ptr() as *const c_char,
    sort_subcommands: true,
    details: &RzCmdDescDetail {
        name: b"\0".as_ptr() as *const c_char,
        entries: ptr::null(),
    } as *const RzCmdDescDetail,
    details_cb: None,
};

const cmd_psp_args: [RzCmdDescArg;1] = {
    [ 
        RzCmdDescArg { 
            name: b"subcommand\0".as_ptr() as *const c_char,
            type_: rz_cmd_arg_type_t_RZ_CMD_ARG_TYPE_STRING,
            optional: false, 
            no_space: false, 
            flags: 0,
            default_value: b"\0".as_ptr() as *const c_char,
            __bindgen_anon_1: unsafe { mem::zeroed() }
        },
    ]
};

fn do_nid_stuff() {
    println!("Hello NID");
}

#[no_mangle]
pub static mut rizin_plugin: RzLibStruct = RzLibStruct {
    type_: RzLibType_RZ_LIB_TYPE_CORE,
    data: &rz_core_plugin_psp as *const _ as *mut c_void, // lolwut?
    version: b"0.6.3\0".as_ptr() as *const _,
    free: None,
};
