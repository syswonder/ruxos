fn main() {
    println!("cargo:rustc-link-lib=lwip");
    println!("cargo:rerun-if-changed=custom");
    println!("cargo:rerun-if-changed=depend");
    println!("cargo:rerun-if-changed=wrapper.h");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let clippy_args = std::env::var("CLIPPY_ARGS");

    // Not build with clippy or doc
    if target_os == "none" && clippy_args.is_err() {
        compile_lwip();
    }
    generate_lwip_bindings();
}

fn generate_lwip_bindings() {
    let bindings = bindgen::Builder::default()
        .use_core()
        .header("wrapper.h")
        .clang_arg("-I./depend/lwip/src/include")
        .clang_arg("-I./custom")
        .clang_arg("-Wno-everything")
        .layout_tests(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings!");
}

fn compile_lwip() {
    let mut base_config = cc::Build::new();

    base_config
        .include("depend/lwip/src/include")
        .include("custom");

    base_config
        .file("depend/lwip/src/api/err.c")
        .file("depend/lwip/src/core/init.c")
        .file("depend/lwip/src/core/def.c")
        .file("depend/lwip/src/core/dns.c")
        .file("depend/lwip/src/core/inet_chksum.c")
        .file("depend/lwip/src/core/ip.c")
        .file("depend/lwip/src/core/mem.c")
        .file("depend/lwip/src/core/memp.c")
        .file("depend/lwip/src/core/netif.c")
        .file("depend/lwip/src/core/pbuf.c")
        .file("depend/lwip/src/core/raw.c")
        .file("depend/lwip/src/core/stats.c")
        .file("depend/lwip/src/core/sys.c")
        .file("depend/lwip/src/core/altcp.c")
        .file("depend/lwip/src/core/altcp_alloc.c")
        .file("depend/lwip/src/core/altcp_tcp.c")
        .file("depend/lwip/src/core/tcp.c")
        .file("depend/lwip/src/core/tcp_in.c")
        .file("depend/lwip/src/core/tcp_out.c")
        .file("depend/lwip/src/core/timeouts.c")
        .file("depend/lwip/src/core/udp.c")
        .file("depend/lwip/src/core/ipv4/autoip.c")
        .file("depend/lwip/src/core/ipv4/dhcp.c")
        .file("depend/lwip/src/core/ipv4/etharp.c")
        .file("depend/lwip/src/core/ipv4/icmp.c")
        .file("depend/lwip/src/core/ipv4/igmp.c")
        .file("depend/lwip/src/core/ipv4/ip4_frag.c")
        .file("depend/lwip/src/core/ipv4/ip4.c")
        .file("depend/lwip/src/core/ipv4/ip4_addr.c")
        .file("depend/lwip/src/netif/ethernet.c")
        .file("custom/custom_pool.c");

    base_config
        .warnings(true)
        .flag("-static")
        .flag("-no-pie")
        .flag("-fno-builtin")
        .flag("-ffreestanding")
        .compile("liblwip.a");
}
