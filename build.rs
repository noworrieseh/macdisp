fn main() {
    // Build the Objective-C helper
    cc::Build::new()
        .file("src/objc/display_services.m")
        .flag("-fobjc-arc")
        .compile("display_services");

    // Link against required frameworks
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=CoreGraphics");
    println!("cargo:rustc-link-lib=framework=AppKit");
    println!("cargo:rustc-link-lib=framework=IOKit");

    // Try to link DisplayServices if available (private framework)
    // This will fail gracefully if not found
    println!("cargo:rustc-link-search=framework=/System/Library/PrivateFrameworks");
    println!("cargo:rustc-link-lib=framework=DisplayServices");

    println!("cargo:rerun-if-changed=src/objc/display_services.m");
    println!("cargo:rerun-if-changed=src/objc/display_services.h");
}
