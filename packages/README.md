# Pacotes locais do Aura OS

Este diretório contém os PKGBUILDs que não estão nos repositórios oficiais do
Arch Linux. O script `build-local-repo.sh` transforma as árvores de trabalho em
pacotes, cria o banco `aura.db.tar.gz` e publica tudo em
`/tmp/aura-os-repo`, o repositório usado durante o build da ISO.

## Pacotes

- `aura-dl`: binário Rust para listar e baixar vídeos/áudios com `yt-dlp` e
  `ffmpeg`.
- `aura-dl-bridge`: host de mensagens nativas e extensão para Firefox/LibreWolf.

## Build manual

Em um host Arch Linux:

```bash
sudo pacman -S --needed base-devel libarchive rust
sudo bash packages/build-local-repo.sh
```

O script não altera a árvore do repositório: os fontes e os pacotes são
construídos em uma pasta temporária. Para limpar o resultado:

```bash
sudo rm -rf /tmp/aura-os-repo
```

O `build.sh` da ISO chama esse script automaticamente. Como o `makepkg` não
pode ser executado como root, quando o script é chamado por `sudo` o build é
delegado ao usuário `nobody`, com caches temporários isolados.
