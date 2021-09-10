// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    env, fs, fmt,
    path::{Path, PathBuf},
    process::Stdio,
    io::prelude::*,
};

use bindgen::RustTarget;
use flapigen::{JavaConfig, JavaReachabilityFence, LanguageConfig};

#[path = "src/foreign_types/attributes.rs"]
mod attributes;

static INCLUDE_SYS_H: [&str; 1] = ["jni.h"];

static ANDROID_TARGETS: &'static [&'static str] = &[
    "aarch64-linux-android",
    "arm-linux-androideabi",
    "armv7-linux-androideabi",
    "i686-linux-android",
    "x86_64-linux-android",
];

fn main() {
    // don't simplify this to if the target contains the substring "android" --
    // these lines also serve as a guard so only true android triples receive
    // JNI generation.
    let target = env::var("TARGET").unwrap();

    env_logger::init();
    let out_dir = env::var("OUT_DIR").unwrap();
    let in_src = Path::new("src").join("java_glue.rs.in");
    let out_src = Path::new(&out_dir).join("java_glue.rs");

    let mut java_cfg = JavaConfig::new(
        Path::new("src")
            .join("main")
            .join("java")
            .join("org")
            .join("iota")
            .join("wallet"),
        "org.iota.wallet".into(),
    );

    if ANDROID_TARGETS.contains(&target.as_str()){
        //java_cfg = java_cfg.use_null_annotation_from_package("androidx.annotation.Nullable".into());
    }

    let swig_gen = flapigen::Generator::new(LanguageConfig::JavaConfig(java_cfg))
        .rustfmt_bindings(true)
        .remove_not_generated_files_from_output_directory(false)
        .merge_type_map("chrono_support", include_str!("src/foreign_types/chrono_include.rs"))
        .merge_type_map("foreign_types", include_str!("src/foreign_types/types.rs"))
        .register_class_attribute_callback("PartialEq", attributes::class_partial_eq)
        .register_class_attribute_callback("Display", attributes::class_to_string);
    swig_gen.expand_many("flapigen_test_jni", &[&in_src], &out_src);

    println!("cargo:rerun-if-changed={}", in_src.display());
    println!("cargo:rerun-if-changed=src/foreign_types/chrono_include.rs");
}

fn gen_jni_bindings(jni_c_headers_rs: &Path) {
    let java_home = env::var("JAVA_HOME").expect("JAVA_HOME env variable not settted");

    let java_include_dir = Path::new(&java_home).join("include");

    let target = env::var("TARGET").expect("target env var not setted");

    let java_sys_include_dir = java_include_dir.join(if target.contains("windows") {
        "win32"
    } else if target.contains("darwin") {
        "darwin"
    } else {
        "linux"
    });
    
    //let include_dirs = [java_include_dir, java_sys_include_dir];
    let include_dirs = get_cc_system_include_dirs().expect("Can't get NDK's system include dirs");
    println!("jni include dirs {:?}", include_dirs);

    let include_headers: Vec<_> = INCLUDE_SYS_H
        .iter()
        .map(|h| {
            search_file_in_directory(&include_dirs, h)
                .expect(format!("Could not find header {}", h).as_ref())
        })
        .collect();

    gen_binding(&target, &include_dirs, &include_headers, &jni_c_headers_rs).expect("gen_binding failed");

    for dir in &include_dirs {
        println!("cargo:rerun-if-changed={}", dir.display());
    }
    println!("cargo:rerun-if-changed={}", &jni_c_headers_rs.display());
}

fn get_cc_system_include_dirs() -> Result<Vec<PathBuf>, String> {
    let cc_build = cc::Build::new();

    let cc_process = cc_build
        .get_compiler()
        .to_command()
        .env("LANG", "C")
        .env("LC_MESSAGES", "C")
        .args(&["-v", "-x", "c", "-E", "-"])
        .stderr(Stdio::piped())
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .spawn()
        .map_err(|err| err.to_string())?;

    cc_process
        .stdin
        .ok_or_else(|| "can not get stdin of cc".to_string())?
        .write_all(b"\n")
        .map_err(|err| err.to_string())?;

    let mut cc_output = String::new();

    cc_process
        .stderr
        .ok_or_else(|| "can not get stderr of cc".to_string())?
        .read_to_string(&mut cc_output)
        .map_err(|err| err.to_string())?;

    const BEGIN_PAT: &str = "\n#include <...> search starts here:\n";
    const END_PAT: &str = "\nEnd of search list.\n";
    let start_includes = cc_output
        .find(BEGIN_PAT)
        .ok_or_else(|| format!("No '{}' in output from C compiler", BEGIN_PAT))?
        + BEGIN_PAT.len();
    let end_includes = (&cc_output[start_includes..])
        .find(END_PAT)
        .ok_or_else(|| format!("No '{}' in output from C compiler", END_PAT))?
        + start_includes;
    Ok((&cc_output[start_includes..end_includes])
        .split('\n')
        .map(|s| PathBuf::from(s.trim().to_string()))
        .collect())
}

fn search_file_in_directory<P: AsRef<Path>>(dirs: &[P], file: &str) -> Result<PathBuf, ()> {
    for dir in dirs {
        let dir = dir.as_ref().to_path_buf();
        let file_path = dir.join(file);
        if file_path.exists() && file_path.is_file() {
            return Ok(file_path);
        }
    }
    Err(())
}

fn gen_binding<P1, P2>(
    target: &str,
    include_dirs: &[P1],
    c_headers: &[P2],
    output_rust: &Path,
) -> Result<(), String>
where
    P1: AsRef<Path> + fmt::Debug,
    P2: AsRef<Path> + fmt::Debug,
{
    assert!(!c_headers.is_empty());
    let c_file_path = &c_headers[0];

    let mut bindings: bindgen::Builder =
        bindgen::builder().header(c_file_path.as_ref().to_str().unwrap());
    bindings = include_dirs.iter().fold(bindings, |acc, x| {
        acc.clang_arg("-I".to_string() + x.as_ref().to_str().unwrap())
    });
    println!("Generate binding for {:?}", c_headers);
    bindings = bindings
        .rust_target(RustTarget::Stable_1_19)
        //long double not supported yet, see https://github.com/servo/rust-bindgen/issues/550
        .blocklist_type("max_align_t");
    bindings = if target.contains("windows") {
        //see https://github.com/servo/rust-bindgen/issues/578
        bindings.trust_clang_mangling(false)
    } else {
        bindings
    };
    bindings = c_headers[1..].iter().fold(
        Ok(bindings),
        |acc: Result<bindgen::Builder, String>, header| {
            let c_file_path = header;
            let c_file_str = c_file_path
                .as_ref()
                .to_str()
                .ok_or_else(|| format!("Invalid unicode in path to {:?}", c_file_path.as_ref()))?;
            Ok(acc.unwrap().clang_arg("-include").clang_arg(c_file_str))
        },
    )?;

    let generated_bindings = bindings
        //        .clang_arg(format!("-target {}", target))
        .generate()
        .map_err(|_| "Failed to generate bindings".to_string())?;
    generated_bindings
        .write_to_file(output_rust)
        .map_err(|err| err.to_string())?;

    Ok(())
}

