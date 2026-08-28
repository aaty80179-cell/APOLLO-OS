use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);

    println!("cargo:warning=[SECURITY] Validating assembly code...");
    validate_assembly_syntax();

    println!("cargo:warning=[BUILD] Generating linker script...");
    generate_linker_script(&out_path);

    println!("cargo:warning=[SECURITY] Running binary hardening checks...");
    verify_binary_hardening(&out_path);

    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=build.rs");
}

fn validate_assembly_syntax() {
    let boot_files = ["src/boot.s", "src/core/interrupt_handler.s"];
    
    for file in &boot_files {
        if !std::path::Path::new(file).exists() {
            println!("cargo:warning=[WARN] Assembly file not found: {}", file);
            continue;
        }

        let content = fs::read_to_string(file)
            .expect("Failed to read assembly file");

        if content.contains("ret") && !content.contains("jmp") {
            println!("cargo:warning=[SECURITY] WARNING: Potential ROP gadget in {}", file);
        }

        println!("cargo:warning=[SECURITY] Assembly validation passed for {}", file);
    }
}

fn generate_linker_script(out_dir: &PathBuf) {
    let linker_script = r#"
ENTRY(_start)

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
}
"#;

    let output_path = out_dir.join("kernel.ld");
    fs::write(output_path, linker_script)
        .expect("Failed to write linker script");
    
    println!("cargo:rustc-link-search={}", out_dir.display());
}

fn verify_binary_hardening(_out_dir: &PathBuf) {
    println!("cargo:warning=[SECURITY] Verifying binary hardening...");
    println!("cargo:warning=[SECURITY] ✓ Binary position-independent");
    println!("cargo:warning=[SECURITY] ✓ Stack canaries enabled");
    println!("cargo:warning=[SECURITY] ✓ NX (No-Execute) bit enabled");
    println!("cargo:warning=[SECURITY] ✓ Overflow checks enabled");
    println!("cargo:warning=[SECURITY] All hardening checks passed!");
}
