use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SiteProfile {
    site_key: String,
    brand_name: String,
    app_name: String,
    default_base_url: String,
    default_model: String,
    store_url: String,
    docs_url: String,
    provider_id: String,
    provider_name: String,
    sqlite_dir_name: String,
    portable_home_dir_name: String,
    export_file_name: String,
}

fn main() {
    println!("cargo:rerun-if-env-changed=SWITCH_SITE");
    println!("cargo:rerun-if-changed=../src/site-profiles.json");

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let profiles_path = manifest_dir.join("../src/site-profiles.json");
    let profiles_raw = fs::read_to_string(&profiles_path).expect("read site profiles");
    let profiles: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&profiles_raw).expect("parse site profiles");

    let site_key = std::env::var("SWITCH_SITE").unwrap_or_else(|_| "hajimi".to_string());
    let site_value = profiles
        .get(&site_key)
        .unwrap_or_else(|| panic!("unknown SWITCH_SITE: {site_key}"))
        .clone();
    let profile: SiteProfile = serde_json::from_value(site_value).expect("decode site profile");

    let generated = format!(
        "pub const SITE_PROFILE: SiteProfile = SiteProfile {{
    site_key: {site_key:?},
    brand_name: {brand_name:?},
    app_name: {app_name:?},
    default_base_url: {default_base_url:?},
    default_model: {default_model:?},
    store_url: {store_url:?},
    docs_url: {docs_url:?},
    provider_id: {provider_id:?},
    provider_name: {provider_name:?},
    sqlite_dir_name: {sqlite_dir_name:?},
    portable_home_dir_name: {portable_home_dir_name:?},
    export_file_name: {export_file_name:?},
}};\n",
        site_key = profile.site_key,
        brand_name = profile.brand_name,
        app_name = profile.app_name,
        default_base_url = profile.default_base_url,
        default_model = profile.default_model,
        store_url = profile.store_url,
        docs_url = profile.docs_url,
        provider_id = profile.provider_id,
        provider_name = profile.provider_name,
        sqlite_dir_name = profile.sqlite_dir_name,
        portable_home_dir_name = profile.portable_home_dir_name,
        export_file_name = profile.export_file_name,
    );

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("out dir"));
    fs::write(out_dir.join("site_profile.rs"), generated).expect("write generated site profile");

    tauri_build::build()
}
