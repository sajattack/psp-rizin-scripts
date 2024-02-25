use core::ffi::{c_char, c_void};

use rizin_librz_sys::RzCorePlugin;
use rizin_librz_sys::rz_core_t;
use rizin_librz_sys::rz_core_plugin_t;
use rizin_librz_sys::RzLibStruct;
use rizin_librz_sys::RzLibType_RZ_LIB_TYPE_CORE;


const rz_core_plugin_psp_resolve_nid: RzCorePlugin = RzCorePlugin {
	name: b"rz-psp-resolve-nid\0".as_ptr() as *const c_char,
	desc: b"NID Plugin for Rizin\0".as_ptr() as *const c_char,
	license: b"MIT\0".as_ptr() as *const c_char,
	version: b"v0.1.0\0".as_ptr() as *const c_char,
	author: b"sajattack\0".as_ptr() as *const c_char,
	init:  Some(rz_cmd_psp_resolve_nid_init),
	fini: None,
	analysis: None,
};

#[no_mangle]
pub unsafe extern "C" fn rz_cmd_psp_resolve_nid_init(p: *mut rz_core_t) -> bool 
{
    println!("Hello World!");
    true    
}

#[no_mangle]
pub static mut rizin_plugin: RzLibStruct = RzLibStruct {
    type_: RzLibType_RZ_LIB_TYPE_CORE,
    data: &rz_core_plugin_psp_resolve_nid as *const _ as *mut c_void, // lolwut?
    version: b"0.6.3\0".as_ptr() as *const _,
    free: None,
};
