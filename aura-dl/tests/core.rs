//! Testes do núcleo. Não dependem de rede nem do yt-dlp instalado: a saída
//! do yt-dlp vem de fixture, e os argumentos são só montados, não executados.

use std::fs;
use std::path::{Path, PathBuf};

use aura_dl::backend::{
    build_download_args, parse_metadata, parse_progress_line, validate_container, DownloadRequest,
    Mode,
};
use aura_dl::input::{sanitize_filename, unique_path, validate_url};
use aura_dl::Error;

const FIXTURE: &str = include_str!("fixtures/sample_metadata.json");

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("aura-dl-test-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("criar pasta temporária");
    dir
}

// --- URLs ---

#[test]
fn aceita_http_e_https() {
    assert!(validate_url("https://example.com/video").is_ok());
    assert!(validate_url("  http://example.com/video  ").is_ok());
}

#[test]
fn recusa_esquemas_perigosos() {
    for raw in ["file:///etc/passwd", "javascript:alert(1)", "ftp://x.org/a"] {
        assert!(
            matches!(validate_url(raw), Err(Error::UnsupportedScheme(_))),
            "deveria recusar {raw}"
        );
    }
}

#[test]
fn recusa_url_malformada() {
    assert!(matches!(
        validate_url("não é uma url"),
        Err(Error::InvalidUrl(_))
    ));
}

// --- Nomes de arquivo ---

#[test]
fn remove_separadores_de_caminho() {
    let limpo = sanitize_filename("../../etc/passwd");
    assert!(!limpo.contains('/'));
    assert!(!limpo.starts_with('.'));
}

#[test]
fn substitui_caracteres_proibidos_no_windows() {
    assert_eq!(sanitize_filename("a:b*c?d"), "a_b_c_d");
}

#[test]
fn titulo_vazio_vira_video() {
    assert_eq!(sanitize_filename("   ..   "), "video");
    assert_eq!(sanitize_filename(""), "video");
}

#[test]
fn limita_tamanho_do_nome() {
    let longo = "á".repeat(500);
    assert_eq!(sanitize_filename(&longo).chars().count(), 200);
}

#[test]
fn nao_sobrescreve_arquivo_existente() {
    let dir = temp_dir("unique");
    fs::write(dir.join("video.mp4"), b"x").unwrap();
    fs::write(dir.join("video (1).mp4"), b"x").unwrap();

    let escolhido = unique_path(&dir, "video", "mp4").unwrap();
    assert_eq!(escolhido, dir.join("video (2).mp4"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn pasta_inexistente_gera_erro() {
    let inexistente = Path::new("/caminho/que/nao/existe/aura");
    assert!(matches!(
        unique_path(inexistente, "x", "mp4"),
        Err(Error::MissingDir(_))
    ));
}

// --- Argumentos do yt-dlp ---

fn req(mode: Mode, container: &str) -> DownloadRequest {
    DownloadRequest {
        url: "https://example.com/v".into(),
        mode,
        container: container.into(),
        max_height: 720,
    }
}

#[test]
fn video_usa_seletor_de_altura_e_merge() {
    let args = build_download_args(&req(Mode::Video, "mp4"), "/tmp/%(title)s.%(ext)s").unwrap();
    assert!(args
        .windows(2)
        .any(|w| w[0] == "-f" && w[1].contains("height<=720")));
    assert!(args
        .windows(2)
        .any(|w| w[0] == "--merge-output-format" && w[1] == "mp4"));
}

#[test]
fn audio_extrai_sem_video() {
    let args = build_download_args(&req(Mode::Audio, "mp3"), "/tmp/%(title)s.%(ext)s").unwrap();
    assert!(args.contains(&"-x".to_owned()));
    assert!(args
        .windows(2)
        .any(|w| w[0] == "--audio-format" && w[1] == "mp3"));
    assert!(!args.contains(&"-f".to_owned()));
}

#[test]
fn url_fica_por_ultimo_e_nunca_vira_comando_de_shell() {
    let args = build_download_args(&req(Mode::Video, "mp4"), "/tmp/x").unwrap();
    assert_eq!(args.last().unwrap(), "https://example.com/v");
}

#[test]
fn recusa_contenior_do_modo_errado() {
    assert!(matches!(
        validate_container(Mode::Audio, "mp4"),
        Err(Error::UnsupportedFormat(_))
    ));
    assert!(matches!(
        validate_container(Mode::Video, "mp3"),
        Err(Error::UnsupportedFormat(_))
    ));
}

// --- Progresso ---

#[test]
fn le_porcentagem_do_progresso() {
    assert_eq!(parse_progress_line("download:  42.5%"), Some(42.5));
    assert_eq!(parse_progress_line("download:100.0%"), Some(100.0));
}

#[test]
fn ignora_linhas_que_nao_sao_progresso() {
    assert_eq!(parse_progress_line("[info] Downloading webpage"), None);
    assert_eq!(parse_progress_line("download:  N/A"), None);
}

// --- Metadados ---

#[test]
fn interpreta_formatos_do_fixture() {
    let meta = parse_metadata(FIXTURE).expect("fixture deve ser JSON válido");
    assert_eq!(meta.formats.len(), 3);

    let so_audio = meta.formats.iter().find(|f| f.format_id == "140").unwrap();
    assert_eq!(so_audio.vcodec.as_deref(), Some("none"));
    assert_eq!(so_audio.height, None);

    let video = meta.formats.iter().find(|f| f.format_id == "137").unwrap();
    assert_eq!(video.height, Some(1080));
    assert_eq!(video.filesize, None);
    assert_eq!(video.filesize_approx, Some(85_000_000));
}

#[test]
fn json_invalido_vira_erro_tipado() {
    assert!(matches!(
        parse_metadata("{não é json"),
        Err(Error::Metadata(_))
    ));
}
