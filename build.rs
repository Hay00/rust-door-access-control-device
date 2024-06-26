#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
    #[default("")]
    mqtt_url: &'static str,
    #[default("")]
    mqtt_port: &'static str,
}

fn main() -> anyhow::Result<()> {
    // Check if the `cfg.toml` file exists and has been filled out.
    if !std::path::Path::new("cfg.toml").exists() {
        anyhow::bail!("You need to create a `cfg.toml` file with your Wi-Fi credentials! Use `cfg.toml.example` as a template.");
    }

    // The constant `CONFIG` is auto-generated by `toml_config`.
    let app_config = CONFIG;
    if app_config.wifi_ssid == "" || app_config.wifi_psk == "" {
        anyhow::bail!("You need to set the Wi-Fi credentials in `cfg.toml`!");
    }

    // Necessary because of this issue: https://github.com/rust-lang/cargo/issues/9641
    embuild::build::CfgArgs::output_propagated("ESP_IDF")?;
    embuild::build::LinkArgs::output_propagated("ESP_IDF")
}
