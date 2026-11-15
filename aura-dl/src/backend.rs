//! Integração com o yt-dlp por subprocesso.
//!
//! Não reimplementamos extratores: o yt-dlp mantém centenas de sites e é
//! atualizado com frequência. Aqui só montamos argumentos (sempre como lista,
//! nunca como string de shell), chamamos o binário e lemos a saída.

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Output, Stdio};

use serde::Deserialize;

use crate::error::{Error, Result};

const YTDLP: &str = "yt-dlp";
const VIDEO_CONTAINERS: &[&str] = &["mp4", "mkv", "webm"];
const AUDIO_FORMATS: &[&str] = &["mp3", "m4a", "opus"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Video,
    Audio,
}

/// Parâmetros de um download já validados. `url` deve ter passado por
/// `input::validate_url` antes de chegar aqui.
#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub mode: Mode,
    pub container: String,
    pub max_height: u32,
}

/// Formato padrão para cada modo. A CLI usa isso quando `--format` é omitido.
pub fn default_container(mode: Mode) -> &'static str {
    match mode {
        Mode::Video => "mp4",
        Mode::Audio => "mp3",
    }
}

/// Confere se o contêiner ou formato de áudio pedido é suportado no modo escolhido.
pub fn validate_container(mode: Mode, container: &str) -> Result<()> {
    let allowed = match mode {
        Mode::Video => VIDEO_CONTAINERS,
        Mode::Audio => AUDIO_FORMATS,
    };
    if allowed.contains(&container) {
        Ok(())
    } else {
        Err(Error::UnsupportedFormat(container.to_owned()))
    }
}

/// Confere se a altura pedida está na faixa aceita pelo downloader.
pub fn validate_quality(max_height: u32) -> Result<()> {
    if (144..=4320).contains(&max_height) {
        Ok(())
    } else {
        Err(Error::InvalidQuality(max_height))
    }
}

/// Monta os argumentos do yt-dlp para um download. A URL vai por último e a
/// saída é controlada pelo `out_template`, que o yt-dlp preenche com o título.
pub fn build_download_args(req: &DownloadRequest, out_template: &str) -> Result<Vec<String>> {
    validate_container(req.mode, &req.container)?;
    validate_quality(req.max_height)?;

    let mut args: Vec<String> = vec![
        "--no-playlist".into(),
        // Nunca substitui um download existente, mesmo se o id do vídeo repetir.
        "--no-overwrites".into(),
        // Mantém o título portátil para sistemas de arquivos Windows.
        "--windows-filenames".into(),
        // Uma linha por atualização de progresso, para lermos de forma incremental.
        "--newline".into(),
        "--progress-template".into(),
        "download:%(progress._percent_str)s".into(),
        "-o".into(),
        out_template.into(),
    ];

    match req.mode {
        Mode::Video => {
            // Melhor vídeo até a altura pedida somado ao melhor áudio; o yt-dlp
            // faz o merge com ffmpeg usando -c copy quando os codecs permitem.
            let selector = format!("bv*[height<={h}]+ba/b[height<={h}]", h = req.max_height);
            args.extend([
                "-f".into(),
                selector,
                "--merge-output-format".into(),
                req.container.clone(),
            ]);
        }
        Mode::Audio => {
            args.extend(["-x".into(), "--audio-format".into(), req.container.clone()]);
        }
    }

    args.push(req.url.clone());
    Ok(args)
}

/// Extrai a porcentagem de uma linha de progresso como `download: 42.5%`.
/// Linhas que não são progresso, ou com `N/A` (comum no início), retornam `None`.
pub fn parse_progress_line(line: &str) -> Option<f32> {
    let rest = line.trim().strip_prefix("download:")?.trim();
    let number = rest.strip_suffix('%')?.trim();
    number.parse::<f32>().ok()
}

/// Metadados de um vídeo, na forma que o `yt-dlp -J` emite.
#[derive(Debug, Deserialize, PartialEq)]
pub struct Metadata {
    pub title: Option<String>,
    #[serde(default)]
    pub formats: Vec<Format>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Format {
    pub format_id: String,
    pub ext: Option<String>,
    pub height: Option<u32>,
    /// `"none"` indica que o formato não tem vídeo (é só áudio).
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
    /// Bitrate médio de áudio em kbps.
    pub abr: Option<f64>,
    pub filesize: Option<u64>,
    pub filesize_approx: Option<u64>,
    pub format_note: Option<String>,
}

/// Interpreta o JSON emitido por `yt-dlp -J`.
pub fn parse_metadata(json: &str) -> Result<Metadata> {
    Ok(serde_json::from_str(json)?)
}

/// Executa o yt-dlp com argumentos separados. Erro claro se o binário não existir.
fn run_ytdlp(args: &[String]) -> Result<Output> {
    Command::new(YTDLP).args(args).output().map_err(spawn_error)
}

fn spawn_error(err: std::io::Error) -> Error {
    if err.kind() == std::io::ErrorKind::NotFound {
        Error::MissingYtDlp
    } else {
        Error::Io(err)
    }
}

/// Lista os formatos disponíveis para uma URL, sem baixar nada.
pub fn fetch_metadata(url: &str) -> Result<Metadata> {
    let args = vec!["-J".to_owned(), "--no-playlist".to_owned(), url.to_owned()];
    let output = run_ytdlp(&args)?;
    if !output.status.success() {
        return Err(Error::YtDlpFailed {
            code: output.status.code(),
        });
    }
    parse_metadata(&String::from_utf8_lossy(&output.stdout))
}

/// Executa o download e chama `on_progress` a cada atualização de porcentagem.
/// A saída de erro do yt-dlp vai direto para o terminal, para não travar o pipe.
pub fn download(args: &[String], mut on_progress: impl FnMut(f32)) -> Result<()> {
    let mut child = Command::new(YTDLP)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(spawn_error)?;

    if let Some(stdout) = child.stdout.take() {
        for line in BufReader::new(stdout).lines() {
            if let Some(percent) = parse_progress_line(&line?) {
                on_progress(percent);
            }
        }
    }

    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::YtDlpFailed {
            code: status.code(),
        })
    }
}

/// Template de saída padrão: título truncado, id para evitar colisão e extensão.
/// Os caracteres inválidos são tratados pelo próprio yt-dlp.
pub fn output_template(dir: &Path) -> String {
    dir.join("%(title).200B [%(id)s].%(ext)s")
        .to_string_lossy()
        .into_owned()
}
