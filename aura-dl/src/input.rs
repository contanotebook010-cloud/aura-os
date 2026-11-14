//! Validação de entrada: URLs, nomes de arquivo e caminhos de saída.
//! Tudo que vem de fora (usuário, página web, título de vídeo) passa por aqui.

use std::path::{Path, PathBuf};

use url::Url;

use crate::error::{Error, Result};

/// Limite de caracteres do nome de arquivo. 200 deixa folga para a extensão e
/// para o sufixo de id do yt-dlp, dentro do limite de 255 bytes do ext4.
const MAX_FILENAME_CHARS: usize = 200;

/// Caracteres proibidos em nomes de arquivo no Linux (`/`) e no Windows
/// (`<>:"\|?*`). Usar a união dos dois mantém os arquivos portáveis para o
/// usuário que copia a pasta para um pendrive.
const INVALID_FILENAME_CHARS: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

/// Aceita somente `http` e `https`. Esquemas como `file:` e `javascript:` são
/// recusados antes de chegar ao subprocesso do yt-dlp.
pub fn validate_url(raw: &str) -> Result<Url> {
    let trimmed = raw.trim();
    let url = Url::parse(trimmed).map_err(|_| Error::InvalidUrl(trimmed.to_owned()))?;
    match url.scheme() {
        "http" | "https" => Ok(url),
        other => Err(Error::UnsupportedScheme(other.to_owned())),
    }
}

/// Transforma um título arbitrário em nome de arquivo seguro.
///
/// Títulos vêm de sites e não são confiáveis: podem conter barras para tentar
/// escapar do diretório, caracteres de controle ou nomes que começam com ponto.
pub fn sanitize_filename(title: &str) -> String {
    let replaced: String = title
        .chars()
        .map(|c| {
            if c.is_control() || INVALID_FILENAME_CHARS.contains(&c) {
                '_'
            } else {
                c
            }
        })
        .collect();

    // Remove pontos e espaços das pontas: "..", " .hidden" e nomes vazios
    // deixariam o arquivo oculto ou apontando para o diretório pai.
    let trimmed = replaced.trim().trim_matches('.').trim();
    let limited: String = trimmed.chars().take(MAX_FILENAME_CHARS).collect();

    if limited.is_empty() {
        "video".to_owned()
    } else {
        limited
    }
}

/// Escolhe um caminho que não sobrescreve arquivo existente:
/// `nome.ext`, depois `nome (1).ext`, `nome (2).ext` e assim por diante.
pub fn unique_path(dir: &Path, stem: &str, ext: &str) -> Result<PathBuf> {
    if !dir.is_dir() {
        return Err(Error::MissingDir(dir.to_path_buf()));
    }

    let first = dir.join(format!("{stem}.{ext}"));
    if !first.exists() {
        return Ok(first);
    }

    (1u32..)
        .map(|n| dir.join(format!("{stem} ({n}).{ext}")))
        .find(|candidate| !candidate.exists())
        .ok_or_else(|| Error::Io(std::io::Error::other("sem nome livre disponível")))
}
