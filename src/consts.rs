use std::path::PathBuf;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub const FILE_EXT: &str = "linux-amd64.tar.gz";
#[cfg(all(target_os = "linux", target_arch = "x86"))]
pub const FILE_EXT: &str = "linux-386.tar.gz";
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub const FILE_EXT: &str = "linux-arm64.tar.gz";
#[cfg(all(target_os = "linux", target_arch = "arm"))]
pub const FILE_EXT: &str = "linux-armv6l.tar.gz";
#[cfg(all(target_os = "linux", target_arch = "powerpc64"))]
pub const FILE_EXT: &str = "linux-ppc64le.tar.gz";
#[cfg(all(target_os = "linux", target_arch = "s390x"))]
pub const FILE_EXT: &str = "linux-s390x.tar.gz";
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub const FILE_EXT: &str = "windows-amd64.zip";
#[cfg(all(target_os = "windows", target_arch = "x86"))]
pub const FILE_EXT: &str = "windows-386.zip";
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub const FILE_EXT: &str = "darwin-amd64.tar.gz";
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub const FILE_EXT: &str = "darwin-arm64.tar.gz";
#[cfg(all(target_os = "freebsd", target_arch = "x86_64"))]
pub const FILE_EXT: &str = "freebsd-amd64.tar.gz";
#[cfg(all(target_os = "freebsd", target_arch = "x86"))]
pub const FILE_EXT: &str = "freebsd-386.tar.gz";

pub const DL_URL: &str = "https://golang.org/dl";

lazy_static! {
	pub static ref CONFIG_DIR: PathBuf = {
        let dirs = directories::ProjectDirs::from("com", "x0f5c3", "Go Manager").unwrap();
		dirs.config_dir().to_path_buf()
	};
	pub static ref CONFIG_PATH: PathBuf = {
        let dirs = directories::ProjectDirs::from("com", "x0f5c3", "Go Manager").unwrap();
		dirs.config_dir().join("config.toml").to_path_buf()
	};
	pub static ref VERSION_LIST: PathBuf = {
        let dirs = directories::ProjectDirs::from("com", "x0f5c3", "Go Manager").unwrap();
		dirs.config_dir().join("versions.toml").to_path_buf()
	};
}