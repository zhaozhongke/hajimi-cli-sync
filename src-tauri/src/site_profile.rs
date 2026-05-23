#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct SiteProfile {
    pub site_key: &'static str,
    pub brand_name: &'static str,
    pub app_name: &'static str,
    pub default_base_url: &'static str,
    pub default_model: &'static str,
    pub store_url: &'static str,
    pub docs_url: &'static str,
    pub provider_id: &'static str,
    pub provider_name: &'static str,
    pub sqlite_dir_name: &'static str,
    pub portable_home_dir_name: &'static str,
    pub export_file_name: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/site_profile.rs"));
