fn main() {
    for (key, value) in std::env::vars() {
        eprintln!("{key}: {value}");
    }
    let mut config = cbindgen::Config::default();
    config.pragma_once = true;
    config.language = cbindgen::Language::C;
    config.documentation_style = cbindgen::DocumentationStyle::Doxy;
    config.cpp_compat = true;
    config.sort_by = cbindgen::SortKey::Name;

    match cbindgen::generate_with_config(".", config) {
        Ok(value) => {
            value.write_to_file("direct_access_capi.h");
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }

    println!("cargo:rerun-if-changed=/src/*");
    println!("cargo:rerun-if-changed=/build.rs");
}
