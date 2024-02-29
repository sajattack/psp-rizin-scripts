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
    rz_io_read_at, rz_io_read_at_mapped, rz_io_size, RzBinSection, rz_bin_get_sections,
    rz_io_p2v,
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
    let bin = unsafe { (*core).bin };
    let sections = unsafe { rz_bin_get_sections(bin) };
    let sections = section_list_to_vec(sections);
    for s in sections {
        let name = unsafe { CStr::from_ptr(s.name).to_str().unwrap()};
        if name == ".rodata.sceModuleInfo" {
            let mut buf = Vec::new();
            buf.resize(s.size.try_into().unwrap(), 0);
            unsafe { rz_io_read_at(io, s.vaddr, buf.as_mut_ptr(), s.size.try_into().unwrap()) };
            // FIXME
            let boxed_slice = buf[..core::mem::size_of::<sceModuleInfo>()].to_vec().into_boxed_slice();
            let boxed_array: Box<[u8; core::mem::size_of::<sceModuleInfo>()]> = boxed_slice.try_into().unwrap();
            let modinfo = unsafe { core::mem::transmute::<[u8; core::mem::size_of::<sceModuleInfo>()], sceModuleInfo>(*boxed_array)};
            println!("{:x?}", modinfo);

            buf = Vec::new();
            buf.resize(modinfo.exports_end_addr as usize - modinfo.exports_addr as usize, 0);
            let exports_vaddr = unsafe { rz_io_p2v(io, modinfo.exports_addr.try_into().unwrap()) };
            unsafe { rz_io_read_at(io, exports_vaddr as u64, buf.as_mut_ptr(), (modinfo.exports_end_addr - modinfo.exports_addr) as i32) };
            let export_bytes = buf;

            println!("export_bytes {:x?}", export_bytes);

            let boxed_slice = export_bytes[..core::mem::size_of::<sceModuleExport>()].to_vec().into_boxed_slice();
            let boxed_array: Box<[u8; core::mem::size_of::<sceModuleExport>()]> = boxed_slice.try_into().unwrap();
            let exports = unsafe { core::mem::transmute::<[u8; core::mem::size_of::<sceModuleExport>()], sceModuleExport>(*boxed_array)};


            //dbg!(exports);
            println!("{:x?}", exports);
            
            buf = Vec::new();
            buf.resize(modinfo.imports_end_addr as usize - modinfo.imports_addr as usize, 0);
            let imports_vaddr = unsafe { rz_io_p2v(io, modinfo.imports_addr.try_into().unwrap()) };
            unsafe { rz_io_read_at(io, imports_vaddr as u64, buf.as_mut_ptr(), (modinfo.imports_end_addr - modinfo.imports_addr) as i32) };
            let import_bytes = buf;
            println!("import_bytes {:x?}", import_bytes);

            let boxed_slice = import_bytes[..core::mem::size_of::<sceModuleImport>()].to_vec().into_boxed_slice();
            let boxed_array: Box<[u8; core::mem::size_of::<sceModuleImport>()]> = boxed_slice.try_into().unwrap();
            let imports = unsafe { core::mem::transmute::<[u8; core::mem::size_of::<sceModuleImport>()], sceModuleImport>(*boxed_array)};

            //dbg!(imports);
            println!("{:x?}", imports);
            let imports_name_vaddr = unsafe { rz_io_p2v(io, imports.name.try_into().unwrap()) };

            // FIXME strings longer than 256 chars?
            let mut buf = [0u8; 256];
            unsafe { rz_io_read_at(io, imports_name_vaddr as u64, buf.as_mut_ptr(), 256);}
            //dbg!(buf);
            let import_name = CStr::from_bytes_until_nul(&buf).unwrap().to_str().unwrap();
            println!("{}", import_name);


            let nid_vaddr = unsafe { rz_io_p2v(io, imports.nid_addr.try_into().unwrap()) };


            //imports.func_count * 4; 
            //let mut buf = Vec::new();
            //buf.resize(imports.func_count as usize * 4, 0);
            //unsafe { rz_io_read_at(io, nid_vaddr as u64, buf.as_mut_ptr(), imports.func_count as i32 * 4); }
            //dbg!(buf);  


        } else if name == ".rodata.sceNid" {
            let mut buf = Vec::new();
            buf.resize(s.size.try_into().unwrap(), 0);
            unsafe { rz_io_read_at(io, s.vaddr, buf.as_mut_ptr(), s.size.try_into().unwrap()) };
            println!("sceNid {:x?}", buf);
        }
        //println!("{}", name);
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct sceModuleInfo {
    flags: u32,
    name: [u8; 28],
    gp_addr: u32,
    exports_addr: u32,
    exports_end_addr: u32,
    imports_addr: u32,
    imports_end_addr: u32
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct sceModuleImport {
    name: u32,
    flags: u32,
    entry_size: u8,
    var_count: u8,
    func_count: u16,
    nid_addr: u32,
    funcs_addr: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct sceModuleExport {
    name: u32,
    flags: u32,
    counts: u32,
    exports: u32
}

#[no_mangle]
pub static mut rizin_plugin: RzLibStruct = RzLibStruct {
    type_: RzLibType_RZ_LIB_TYPE_CORE,
    data: &rz_core_plugin_psp as *const _ as *mut c_void, // lolwut?
    version: b"0.6.3\0".as_ptr() as *const _,
    free: None,
};


