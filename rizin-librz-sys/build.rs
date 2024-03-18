use meson_next;
use bindgen::callbacks::{ParseCallbacks, MacroParsingBehavior};
use std::env;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

const IGNORE_MACROS
: [&str; 20] = [
    "FE_DIVBYZERO",
    "FE_DOWNWARD",
    "FE_INEXACT",
    "FE_INVALID",
    "FE_OVERFLOW",
    "FE_TONEAREST",
    "FE_TOWARDZERO",
    "FE_UNDERFLOW",
    "FE_UPWARD",
    "FP_INFINITE",
    "FP_INT_DOWNWARD",
    "FP_INT_TONEAREST",
    "FP_INT_TONEARESTFROMZERO",
    "FP_INT_TOWARDZERO",
    "FP_INT_UPWARD",
    "FP_NAN",
    "FP_NORMAL",
    "FP_SUBNORMAL",
    "FP_ZERO",
    "IPPORT_RESERVED",
];

#[derive(Debug)]
struct IgnoreMacros(HashSet<String>);

impl ParseCallbacks for IgnoreMacros {
    fn will_parse_macro(&self, name: &str) -> MacroParsingBehavior {
        if self.0.contains(name) {
            MacroParsingBehavior::Ignore
        } else {
            MacroParsingBehavior::Default
        }
    }
}

impl IgnoreMacros {
    fn new() -> Self {
        Self(IGNORE_MACROS
            .into_iter().map(|s| s.to_owned()).collect())
    }
}

fn main() {
    let build_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let build_path = build_path.join("build");
    let config = meson_next::config::Config::new().options(HashMap::from([
            ("default_library", "static"),
            ("static_runtime", "true"),
            ("cli", "disabled"),
            ("enable_tests", "false"),
            // The libzip shipped with rizin 0.6.3 doesn't compile
            //("use_sys_libzip", "enabled")
    ]));

    meson_next::build("rizin", build_path.to_str().unwrap(), config);


    println!("cargo:rustc-link-search=native={}", build_path.to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("syscall").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("core").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("sign").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("bin").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("config").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("asm").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("flag").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("bp").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("lang").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("parse").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("main").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("diff").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("cons").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("demangler").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("io").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("reg").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("type").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("magic").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("search").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("hash").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("il").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("crypto").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("util").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("analysis").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("debug").to_str().unwrap());
    println!("cargo:rustc-link-search=native={}", build_path.join("egg").to_str().unwrap());
    println!("cargo:rustc-link-lib=rz_analysis");
    println!("cargo:rustc-link-lib=rz_asm");
    println!("cargo:rustc-link-lib=rz_bin");
    println!("cargo:rustc-link-lib=rz_bp");
    println!("cargo:rustc-link-lib=rz_config");
    println!("cargo:rustc-link-lib=rz_cons");
    println!("cargo:rustc-link-lib=rz_core");
    println!("cargo:rustc-link-lib=rz_crypto");
    println!("cargo:rustc-link-lib=rz_debug");
    println!("cargo:rustc-link-lib=rz_demangler");
    println!("cargo:rustc-link-lib=rz_diff");
    println!("cargo:rustc-link-lib=rz_egg");
    println!("cargo:rustc-link-lib=rz_flag");
    println!("cargo:rustc-link-lib=rz_hash");
    println!("cargo:rustc-link-lib=rz_il");
    println!("cargo:rustc-link-lib=rz_io");
    println!("cargo:rustc-link-lib=rz_lang");
    println!("cargo:rustc-link-lib=rz_magic");
    println!("cargo:rustc-link-lib=rz_main");
    println!("cargo:rustc-link-lib=rz_parse");
    println!("cargo:rustc-link-lib=rz_reg");
    println!("cargo:rustc-link-lib=rz_search");
    println!("cargo:rustc-link-lib=rz_sign");
    //println!("cargo:rustc-link-liibrz_socket");
    println!("cargo:rustc-link-lib=rz_syscall");
    println!("cargo:rustc-link-lib=rz_type");
    println!("cargo:rustc-link-lib=rz_util");
    

    // The input header we would like to generate
    // bindings for.
    let bindings = bindgen::Builder::default()
        .clang_arg(String::from("-I") + &build_path.to_str().unwrap())
        .clang_arg(String::from("-I") + "rizin/librz/include/")
        .clang_arg(String::from("-I") + "rizin/librz/bin/format/elf/")
        .clang_arg(String::from("-I") + "rizin/librz/util/sdb/src/")
        .size_t_is_usize(false)
        .header(PathBuf::from(&build_path).join("rz_userconf.h").to_str().unwrap())
        .header("rizin/librz/include/rz_types.h")
        .header("rizin/librz/include/rz_agraph.h")
        .header("rizin/librz/include/rz_analysis.h")
        .header("rizin/librz/include/rz_asm.h")
        .header("rizin/librz/include/rz_basefind.h")
        .header("rizin/librz/include/rz_bin.h")
        .header("rizin/librz/bin/format/elf/elf.h")
        .header("rizin/librz/include/rz_bind.h")
        .header("rizin/librz/include/rz_bin_dwarf.h")
        .header("rizin/librz/include/rz_cmd.h")
        .header("rizin/librz/include/rz_cmp.h")
        .header("rizin/librz/include/rz_config.h")
        .header("rizin/librz/include/rz_cons.h")
        .header("rizin/librz/include/rz_constructor.h")
        .header("rizin/librz/include/rz_core.h")
        .header("rizin/librz/include/rz_crypto.h")
        .header("rizin/librz/include/rz_debug.h")
        .header("rizin/librz/include/rz_demangler.h")
        .header("rizin/librz/include/rz_diff.h")
        .header("rizin/librz/include/rz_drx.h")
        .header("rizin/librz/include/rz_egg.h")
        .header("rizin/librz/include/rz_endian.h")
        .header("rizin/librz/include/rz_flag.h")
        .header("rizin/librz/include/rz_flirt.h")
        .header("rizin/librz/include/rz_getopt.h")
        .header("rizin/librz/include/rz_hash.h")
        .header("rizin/librz/include/rz_heap_glibc.h")
        //.header("rizin/librz/include/rz_heap_jemalloc.h")
        .header("rizin/librz/include/rz_il.h")
        .header("rizin/librz/include/rz_io.h")
        .header("rizin/librz/include/rz_lang.h")
        .header("rizin/librz/include/rz_lib.h")
        .header("rizin/librz/include/rz_list.h")
        .header("rizin/librz/include/rz_magic.h")
        .header("rizin/librz/include/rz_main.h")
        .header("rizin/librz/include/rz_parse.h")
        .header("rizin/librz/include/rz_pdb.h")
        .header("rizin/librz/include/rz_platform.h")
        .header("rizin/librz/include/rz_project.h")
        .header("rizin/librz/include/rz_reg.h")
        .header("rizin/librz/include/rz_search.h")
        .header("rizin/librz/include/rz_sign.h")
        .header("rizin/librz/include/rz_skiplist.h")
        //.header("rizin/librz/include/rz_socket.h")
        .header("rizin/librz/include/rz_syscall.h")
        .header("rizin/librz/include/rz_vector.h")
        .derive_default(true)
        .derive_debug(true)
        .allowlist_function("rz_.*")
        .allowlist_type("[Rz|rz].*")
        .parse_callbacks(Box::new(IgnoreMacros::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
}
