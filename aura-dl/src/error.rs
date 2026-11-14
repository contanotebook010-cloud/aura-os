//! Erros do núcleo. Tipados com `thiserror` para que a interface e o CLI
//! possam reagir a cada caso; `anyhow` fica só na borda do binário.

use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("URL inválida: {0}")]
    InvalidUrl(String),

    #[error("esquema não suportado ({0}); use http ou https")]
    UnsupportedScheme(String),

    #[error("formato de saída não suportado: {0}")]
    UnsupportedFormat(String),

    #[error("yt-dlp não encontrado no PATH. Instale com: sudo pacman -S yt-dlp")]
    MissingYtDlp,

    #[error("o yt-dlp terminou com erro (código {code:?}); veja a saída acima")]
    YtDlpFailed { code: Option<i32> },

    #[error("não consegui interpretar os metadados do yt-dlp: {0}")]
    Metadata(#[from] serde_json::Error),

    #[error("pasta de destino não existe: {0}")]
    MissingDir(PathBuf),

    #[error("erro de entrada e saída: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
