//! 发版产物的签名工具。
//!
//!   qingjian-release-sign keygen --out <私钥文件>        # 生成密钥对：私钥写文件（不打印），公钥打印出来填进 PUBLIC_KEYS
//!   qingjian-release-sign sign <releases.json>           # 私钥取环境变量 QINGJIAN_INDEX_SIGNING_KEY，写 <文件>.sig
//!   qingjian-release-sign verify <releases.json>         # 按本文件里的 PUBLIC_KEYS 验 <文件>.sig
//!
//! 私钥与公钥都是 base64 的 32 字节，签名是 base64 的 64 字节。
//!
//! 客户端不再内置这套验签（自动更新已移除），本工具只用于自己核对发布出去的签名是否配套。

use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use clap::{Parser, Subcommand};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use thiserror::Error;

const KEY_ENV: &str = "QINGJIAN_INDEX_SIGNING_KEY";

/// 信任的发版公钥（base64 的 32 字节 ed25519 公钥）；换钥时新旧并列一段时间。私钥在 CI 密钥 `QINGJIAN_INDEX_SIGNING_KEY` 里。
const PUBLIC_KEYS: &[&str] = &["2qNCMwKGMzaBOsHQLHUornqcopNzmrVBKAmICEl9rkk="];

#[derive(Parser)]
#[command(name = "qingjian-release-sign")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 生成密钥对
    Keygen {
        /// 私钥写到这个文件（已存在就拒绝）
        #[arg(long)]
        out: PathBuf,
    },

    /// 给文件生成分离签名 <文件>.sig
    Sign { file: PathBuf },

    /// 按内置公钥验 <文件>.sig
    Verify { file: PathBuf },
}

#[derive(Debug, Error)]
enum SignError {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{path} already exists")]
    KeyExists { path: PathBuf },

    #[error("{KEY_ENV} is not set")]
    MissingKey,

    #[error("{KEY_ENV} is not a base64 32-byte key")]
    MalformedKey,

    #[error("signing key does not match any public key built into this tool")]
    ForeignKey,

    #[error("signature is not a base64 64-byte value")]
    MalformedSignature,

    #[error("signature does not match any trusted public key")]
    UntrustedSignature,
}

fn main() {
    if let Err(error) = run(Args::parse().command) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run(command: Command) -> Result<(), SignError> {
    match command {
        Command::Keygen { out } => keygen(&out),
        Command::Sign { file } => sign(&file),
        Command::Verify { file } => {
            let signature = std::fs::read_to_string(signature_path(&file))?;
            verify(&std::fs::read(&file)?, &signature)?;
            println!("{}: signature ok", file.display());
            Ok(())
        }
    }
}

/// 验分离签名：`signature` 是 `<文件>.sig` 的内容（base64 的 64 字节），任何一把信任的公钥验过就算数。
fn verify(message: &[u8], signature: &str) -> Result<(), SignError> {
    let bytes = STANDARD
        .decode(signature.trim())
        .map_err(|_| SignError::MalformedSignature)?;
    let signature = Signature::from_slice(&bytes).map_err(|_| SignError::MalformedSignature)?;
    let trusted = PUBLIC_KEYS.iter().filter_map(|key| {
        let bytes: [u8; 32] = STANDARD.decode(key).ok()?.try_into().ok()?;
        VerifyingKey::from_bytes(&bytes).ok()
    });
    for key in trusted {
        if key.verify_strict(message, &signature).is_ok() {
            return Ok(());
        }
    }
    Err(SignError::UntrustedSignature)
}

fn keygen(out: &Path) -> Result<(), SignError> {
    if out.exists() {
        return Err(SignError::KeyExists {
            path: out.to_owned(),
        });
    }
    let mut seed = [0u8; 32];
    fill_random(&mut seed)?;
    let key = SigningKey::from_bytes(&seed);
    write_private(out, &STANDARD.encode(seed))?;
    println!("private key: {}", out.display());
    println!(
        "public key:  {}",
        STANDARD.encode(key.verifying_key().to_bytes())
    );
    Ok(())
}

fn sign(file: &Path) -> Result<(), SignError> {
    let encoded = std::env::var(KEY_ENV).map_err(|_| SignError::MissingKey)?;
    let seed: [u8; 32] = STANDARD
        .decode(encoded.trim())
        .ok()
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or(SignError::MalformedKey)?;
    let key = SigningKey::from_bytes(&seed);
    // 私钥与程序里的公钥对不上时签出来的索引所有客户端都不认，当场拦下
    let public = STANDARD.encode(key.verifying_key().to_bytes());
    if !PUBLIC_KEYS.contains(&public.as_str()) {
        return Err(SignError::ForeignKey);
    }
    let signature = STANDARD.encode(key.sign(&std::fs::read(file)?).to_bytes());
    let out = signature_path(file);
    std::fs::write(&out, format!("{signature}\n"))?;
    println!("{}", out.display());
    Ok(())
}

fn signature_path(file: &Path) -> PathBuf {
    let mut name = file.as_os_str().to_owned();
    name.push(".sig");
    PathBuf::from(name)
}

/// 系统随机源：Unix 读 /dev/urandom；Windows 上不生成密钥（密钥在维护者的 Mac 上生成一次）。
fn fill_random(seed: &mut [u8; 32]) -> Result<(), SignError> {
    use std::io::Read;
    std::fs::File::open("/dev/urandom")?.read_exact(seed)?;
    Ok(())
}

fn write_private(path: &Path, text: &str) -> Result<(), SignError> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)?;
        file.write_all(text.as_bytes())?;
    }
    #[cfg(not(unix))]
    std::fs::write(path, text)?;
    Ok(())
}
