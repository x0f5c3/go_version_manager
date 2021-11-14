use crate::GoVersions;
use std::path::PathBuf;
use versions::SemVer;

pub const DL_URL: &str = "https://golang.org/dl";

#[cfg(target_os = "windows")]
pub const PATH_SEPERATOR: &str = ";";

#[cfg(not(target_os = "windows"))]
pub const PATH_SEPERATOR: &str = ":";

lazy_static! {
    pub static ref FILE_EXT: String = {
        let os = match std::env::consts::OS {
            "windows" => "windows",
            "macos" => "darwin",
            "linux" => "linux",
            "freebsd" => "freebsd",
            x => panic!("OS {} not supported", x),
        };
        let arch = match std::env::consts::ARCH {
            "x86_64" => "amd64",
            "x86" => "386",
            "aarch64" => "arm64",
            "arm" => "armv6l",
            "powerpc64" => "ppc64le",
            "s390x" => "s390x",
            x => panic!("ARCH {} not supported", x),
        };
        let ext = match os {
            "windows" => "zip",
            _ => "tar.gz",
        };
        format!("{}-{}.{}", os, arch, ext)
    };
    pub static ref CONFIG_DIR: PathBuf = {
        let dirs = directories::ProjectDirs::from("rs", "", "Go Manager").unwrap();
        let res = dirs.config_dir().to_path_buf();
        if !res.exists() {
            std::fs::create_dir_all(&res).unwrap();
        }
        res
    };
    pub static ref CLIENT: manic::Client = manic::Client::new();
    pub static ref DL_PAGE: String = CLIENT.get(DL_URL).send().unwrap().text().unwrap();
    pub static ref CONFIG_PATH: PathBuf = CONFIG_DIR.join("config.toml");
    pub static ref VERSION_LIST: PathBuf = CONFIG_DIR.join("versions.toml");
    pub static ref DEFAULT_INSTALL: PathBuf = {
        if cfg!(windows) {
            PathBuf::from("C:\\Go")
        } else {
            PathBuf::from("/usr/local/go")
        }
    };
    pub static ref GIT_VERSIONS: Vec<SemVer> = {
        let output = GoVersions::raw_git_versions().unwrap();
        GoVersions::parse_versions(output).unwrap()
    };
}
