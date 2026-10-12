//! 数据文件位置：只读数据在 `.app/Contents/Resources/`，用户数据在 `~/Library/Application Support/Zha/`。

use std::path::PathBuf;

use objc2_foundation::NSBundle;

use crate::error::HostError;

/// 主 bundle 的 Resources 目录。
pub fn resources_dir() -> Result<PathBuf, HostError> {
    NSBundle::mainBundle()
        .resourcePath()
        .map(|p| PathBuf::from(p.to_string()))
        .ok_or(HostError::NoResources)
}

/// 某个资源文件的完整路径，不存在时报错而不是等到读取时才炸。
pub fn resource(name: &str) -> Result<PathBuf, HostError> {
    let path = resources_dir()?.join(name);
    if path.is_file() {
        Ok(path)
    } else {
        Err(HostError::MissingResource(path))
    }
}

/// 随包的领域词库目录：`.app/Contents/Resources/dicts/`；包里没有就是 `None`。
pub fn bundled_dicts_dir() -> Option<PathBuf> {
    let dir = resources_dir().ok()?.join("dicts");
    dir.is_dir().then_some(dir)
}

/// 形码码表（五笔）：用户目录 `wubi/wubi86.tsv` 里有就用它（自己换的表），
/// 否则用包里的 `Resources/wubi/wubi86.tsv`；都没有是 `None`。
pub fn code_table_path() -> Option<PathBuf> {
    let user = user_data_dir()?.join("wubi/wubi86.tsv");
    if user.is_file() {
        return Some(user);
    }
    let bundled = resources_dir().ok()?.join("wubi/wubi86.tsv");
    bundled.is_file().then_some(bundled)
}

/// 配置文件：`~/Library/Application Support/Zha/config.toml`。
pub fn config_file() -> Option<PathBuf> {
    user_data_dir().map(|dir| dir.join("config.toml"))
}

/// 改名之前的数据目录。还在就说明是从「青简」升上来的，迁移用。
const LEGACY_DATA_DIR: &str = "Qingjian";

/// 用户数据目录，不存在则创建。
///
/// 从青简改名而来时，`Qingjian/` 里还留着配置与学习数据（个人 n-gram、词频、词汇分级）。
/// 首次启动把整个目录挪过来，学习数据不至于归零；挪不动就留在原地继续用旧目录，
/// 免得因为权限问题把用户数据搞丢。
pub fn user_data_dir() -> Option<PathBuf> {
    let home = PathBuf::from(std::env::var_os("HOME")?);
    let support = home.join("Library/Application Support");
    let dir = support.join("Zha");
    if !dir.exists() {
        let legacy = support.join(LEGACY_DATA_DIR);
        if legacy.is_dir() && std::fs::rename(&legacy, &dir).is_ok() {
            tracing::info!(from = %legacy.display(), to = %dir.display(), "数据目录已从青简迁移");
        }
    }
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

/// 用户主题目录：`~/Library/Application Support/Zha/themes/`，一个主题一个子目录（`<id>/theme.json`），不存在则创建。
pub fn themes_dir() -> Option<PathBuf> {
    let dir = user_data_dir()?.join("themes");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

/// 附加词库目录：`~/Library/Application Support/Zha/dicts/`，不存在则创建。
pub fn dicts_dir() -> Option<PathBuf> {
    let dir = user_data_dir()?.join("dicts");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}
