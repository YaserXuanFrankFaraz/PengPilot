//! Makes Cargo rebuild when the locale catalog rust-i18n's proc macro reads
//! changes. Database migrations are embedded by pengpilot-core's build.rs.

fn main() {
    println!("cargo:rerun-if-changed=locales");
}
