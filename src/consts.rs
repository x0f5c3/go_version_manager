use git2::{Direction, Remote};
use std::path::PathBuf;
use versions::Versioning;

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

#[cfg(target_os = "windows")]
pub const PATH_SEPERATOR: &str = ";";

#[cfg(not(target_os = "windows"))]
pub const PATH_SEPERATOR: &str = ":";

lazy_static! {
    pub static ref CONFIG_DIR: PathBuf = {
        let dirs = directories::ProjectDirs::from("com", "x0f5c3", "Go Manager").unwrap();
        let res = dirs.config_dir().to_path_buf();
        if !res.exists() {
            std::fs::create_dir_all(&res).unwrap();
        }
        res
    };
    pub static ref CONFIG_PATH: PathBuf = CONFIG_DIR.join("config.toml");
    pub static ref VERSION_LIST: PathBuf = CONFIG_DIR.join("versions.toml");
    pub static ref DEFAULT_INSTALL: PathBuf = {
        if cfg!(windows) {
            PathBuf::from("C:\\Go")
        } else {
            PathBuf::from("/usr/local/go")
        }
    };
    pub static ref GIT_VERSIONS: Vec<Versioning> = {
        let mut remote = Remote::create_detached("https://github.com/golang/go").unwrap();
        let conn = remote.connect_auth(Direction::Fetch, None, None).unwrap();
        let output: Vec<String> = conn
            .list()
            .unwrap()
            .iter()
            .map(|x| x.name().to_string())
            .filter(|x| x.starts_with("refs/tags/go"))
            .map(|x| x.replace("refs/tags/go", ""))
            .collect();
        let mut parsed: Vec<Versioning> = output
            .iter()
            .filter_map(|x| Versioning::new(x.as_ref()))
            .filter(|x| x.is_ideal())
            .collect();
        parsed.sort_unstable_by(|a, b| b.cmp(a));
        parsed
    };
}
