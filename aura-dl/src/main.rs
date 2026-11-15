//! Binário do aura-dl. Nesta versão só existe o modo `--cli`; a interface
//! gráfica (GTK4 + libadwaita) ainda não foi implementada.

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use aura_dl::backend::{self, DownloadRequest, Mode};
use aura_dl::input;
use clap::{Parser, ValueEnum};
use directories::UserDirs;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ModeArg {
    Video,
    Audio,
}

#[derive(Debug, Parser)]
#[command(
    name = "aura-dl",
    version,
    about = "Baixa vídeos e áudios via yt-dlp e ffmpeg"
)]
struct Cli {
    /// URL do vídeo (http ou https)
    url: Option<String>,

    /// Executa sem interface gráfica
    #[arg(long)]
    cli: bool,

    /// Lista os formatos disponíveis em vez de baixar
    #[arg(long)]
    list: bool,

    /// Modo: vídeo (com áudio) ou só áudio
    #[arg(long, value_enum, default_value_t = ModeArg::Video)]
    mode: ModeArg,

    /// Formato de saída: mp4, mkv, webm para vídeo; mp3, m4a, opus para áudio
    ///
    /// O padrão é mp4 no modo vídeo e mp3 no modo áudio.
    #[arg(long)]
    format: Option<String>,

    /// Altura máxima do vídeo em pixels
    #[arg(long, default_value_t = 1080)]
    quality: u32,

    /// Pasta de destino. Padrão: a pasta de downloads do usuário
    #[arg(long)]
    out: Option<PathBuf>,
}

fn default_download_dir() -> PathBuf {
    UserDirs::new()
        .and_then(|dirs| dirs.download_dir().map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Downloads")))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("AURA_DL_LOG").unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    if !cli.cli {
        bail!("a interface gráfica ainda não existe nesta versão; use --cli");
    }

    let raw_url = cli.url.as_deref().context("informe a URL do vídeo")?;
    let url = input::validate_url(raw_url)?;
    let url = url.as_str();

    if cli.list {
        let meta = backend::fetch_metadata(url)?;
        println!("{}", meta.title.as_deref().unwrap_or("(sem título)"));
        for f in &meta.formats {
            println!(
                "{:<12} {:<6} altura={:<6} vídeo={:<12} áudio={:<12} {}",
                f.format_id,
                f.ext.as_deref().unwrap_or("-"),
                f.height
                    .map(|h| h.to_string())
                    .unwrap_or_else(|| "-".into()),
                f.vcodec.as_deref().unwrap_or("-"),
                f.acodec.as_deref().unwrap_or("-"),
                f.format_note.as_deref().unwrap_or(""),
            );
        }
        return Ok(());
    }

    let mode = match cli.mode {
        ModeArg::Video => Mode::Video,
        ModeArg::Audio => Mode::Audio,
    };
    let out_dir = cli.out.unwrap_or_else(default_download_dir);
    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("não consegui criar a pasta {}", out_dir.display()))?;

    let container = cli
        .format
        .unwrap_or_else(|| backend::default_container(mode).to_owned());
    let request = DownloadRequest {
        url: url.to_owned(),
        mode,
        container,
        max_height: cli.quality,
    };
    let args = backend::build_download_args(&request, &backend::output_template(&out_dir))?;

    backend::download(&args, |percent| {
        eprint!("\r{percent:5.1}%");
    })?;
    eprintln!();
    println!("concluído: {}", out_dir.display());
    Ok(())
}
