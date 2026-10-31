//! Keep AppImage's WebKit renderer away from host color fonts.
//! The policy affects only this process and its WebKit children.
#[cfg(target_os = "linux")]
use std::{fs::OpenOptions, io::Write, os::unix::fs::OpenOptionsExt, path::PathBuf};

#[cfg(target_os = "linux")]
const POLICY: &str = r#"<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd">
<fontconfig>
  <include ignore_missing="yes">/etc/fonts/fonts.conf</include>
  <selectfont><rejectfont><pattern>
    <patelt name="color"><bool>true</bool></patelt>
  </pattern></rejectfont></selectfont>
</fontconfig>
"#;

#[cfg(target_os = "linux")]
pub struct FontPolicy(PathBuf);

#[cfg(target_os = "linux")]
impl Drop for FontPolicy {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Called before GTK or any worker threads are initialized.
#[cfg(target_os = "linux")]
pub fn configure() -> std::io::Result<Option<FontPolicy>> {
    if std::env::var_os("APPIMAGE").is_none() {
        return Ok(None);
    }
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "vibe-timer-fonts-{}-{}.conf",
        std::process::id(),
        nonce
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)?;
    let policy = FontPolicy(path);
    file.write_all(POLICY.as_bytes())?;
    std::env::set_var("FONTCONFIG_FILE", &policy.0);
    Ok(Some(policy))
}
