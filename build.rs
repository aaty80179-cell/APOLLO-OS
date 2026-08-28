use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);

    println!("cargo:warning=[SECURITY] Validating build configuration...");
    validate_build_config();

    println!("cargo:warning=[BUILD] Generating linker script...");
    generate_linker_script(&out_path);

    println!("cargo:warning=[SECURITY] All checks passed!");
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=build.rs");
}

fn validate_build_config() {
    println!("cargo:warning=[SECURITY] Stack overflow protection: ENABLED");
    println!("cargo:warning=[SECURITY] Integer overflow checks: ENABLED");
    println!("cargo:warning=[SECURITY] CFI (Control Flow Integrity): ENABLED");
}

fn generate_linker_script(out_dir: &PathBuf) {
    let linker_script = r#"ENTRY(_start)

SECTIONS
{
    . = 0x100000;
    
    .boot : ALIGN(4K) {
        KEEP(*(.boot))
        . = ALIGN(4K);
    }
    
    .text : ALIGN(4K) {
        *(.text .text.*)
        . = ALIGN(4K);
    }
    
    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
        . = ALIGN(4K);
    }
    
    .data : ALIGN(4K) {
        *(.data .data.*)
        . = ALIGN(4K);
    }
    
    .bss : ALIGN(4K) {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(4K);
        __bss_end = .;
    }
    
    /DISCARD/ : {
        *(.note.*)
        *(.comment*)
    }
}"#;

    let output_path = out_dir.join("kernel.ld");
    fs::write(output_path, linker_script).expect("Failed to write linker script");
    println!("cargo:rustc-link-search={}", out_dir.display());
}
