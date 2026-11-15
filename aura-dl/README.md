# aura-dl

Downloader de vídeo e áudio do Aura OS. Usa o **yt-dlp** para extrair e o **ffmpeg** para o merge.

## Estado atual

- **Pronto:** núcleo (validação de URL, nomes de arquivo seguros, metadados, argumentos do yt-dlp, leitura de progresso) e CLI com `--cli`, `--list`, `--mode`, `--format`, `--quality` e `--out`.
- **Na ISO:** o binário é empacotado por `packages/aura-dl/PKGBUILD` e instalado em `/usr/bin/aura-dl`.
- **Não existe ainda:** interface gráfica GTK4 + libadwaita, fila de downloads, cancelamento, histórico em SQLite, arquivo de configuração TOML e notificações.

## Uso

```bash
aura-dl --cli --list https://exemplo.com/video
aura-dl --cli https://exemplo.com/video --mode video --format mp4 --quality 1080 --out ~/Vídeos
aura-dl --cli https://exemplo.com/video --mode audio --format mp3
```

Progresso é lido da saída do yt-dlp (`--newline` com `--progress-template`), sem valores falsos.

## Decisões

- **yt-dlp por subprocesso:** não reimplementamos extratores. O yt-dlp mantém centenas de sites e é atualizado com frequência.
- **Argumentos em lista:** nunca montamos string de shell. A URL é um argumento separado, o que evita injeção.
- **Só http e https:** outros esquemas são recusados antes de chegar ao yt-dlp.
- **Nomes seguros:** títulos de sites são sanitizados e o yt-dlp usa um template com id, então nada sobrescreve arquivo existente.
- **thiserror no núcleo, anyhow no binário:** erros tipados onde a interface precisa reagir a cada caso.

## Testes

```bash
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo fmt --check
```

Os testes não precisam de rede nem do yt-dlp instalado. A saída de metadados vem de `tests/fixtures/sample_metadata.json`, que foi escrito à mão no formato do `yt-dlp -J`, e não copiado de uma execução real. Quando houver um yt-dlp disponível, substitua o fixture por uma saída real.

## Limitações

- Os testes rodam no CI (Linux), não no notebook Windows, porque o Windows precisa do MSVC Build Tools para linkar.
- Cancelamento de download ainda não está implementado.
- Na ISO, o downloader usa somente a CLI; a ponte do navegador depende de uma extensão temporária até que ela seja assinada pelo Mozilla.
