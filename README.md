# Aura OS

Distribuição live baseada em Arch Linux, com XFCE4 em um preset clássico
inspirado no Windows 10, tema Materia, ícones Papirus e o downloader `aura-dl`
(em Rust).

## Status

Em desenvolvimento. O perfil `archiso`, o pipeline de build e a validação de
boot em QEMU estão implementados. O preset XFCE e o instalador gráfico
Calamares são compilados e verificados durante o pipeline:

- interface gráfica GTK4/libadwaita do `aura-dl`.

O `aura-dl` em modo CLI e a ponte para Firefox/LibreWolf já são empacotados
como repositório local e instalados na ISO.

O instalador Calamares 3.4.3 é compilado do código upstream com checksum
fixo; seus pacotes e configurações são publicados somente no repositório
temporário `/tmp/aura-os-repo` durante a geração da ISO.

## Requisitos para build

Em um host Arch Linux:

```bash
sudo pacman -S --needed archiso base-devel libarchive rust
```

O workflow de GitHub Actions também constrói a ISO em `archlinux:latest`, sem
exigir um host Arch local.

## Como gerar a ISO localmente

```bash
cd iso
sudo ./build.sh
```

A ISO sai em `iso/out/`, com o checksum SHA-256 ao lado. O build exige root
porque o `mkarchiso` monta sistemas de arquivos.

Antes do `mkarchiso`, `packages/build-local-repo.sh` monta os pacotes próprios
do Aura OS e cria o banco em `/tmp/aura-os-repo`. Nenhum arquivo gerado é
gravado na árvore do repositório.

## Como testar

- **QEMU/KVM (BIOS e UEFI):**
  `qemu-system-x86_64 -enable-kvm -m 4G -cdrom iso/out/aura-os-*.iso`
- **VMware Workstation/Player:** crie uma VM Linux 64 bits, aponte o CD/DVD
  para a ISO e ative aceleração 3D.

O primeiro login aplica um preset XFCE idempotente em `/etc/skel` e no usuário
live. Ele configura painel inferior, menu Iniciar Whisker com favoritos,
recentes e busca, lista de janelas, bandeja, relógio, Materia, Papirus, Roboto,
JetBrains Mono, papel de parede original do Aura e atalhos para Este
Computador, Arquivos, Downloads, Documentos, Lixeira e Instalar Aura OS.

O CI inicializa a ISO em QEMU headless e só a aprova quando existe uma sessão
X11 XFCE real do usuário `aura`, o preset está aplicado e validado, e o
Calamares 3.4.3 abre com seus módulos, sequência e configuração presentes. A
live publica `AURA_XFCE_PRESET_OK`, `AURA_CALAMARES_OK` e
`AURA_XFCE_SMOKE_OK` somente depois das respectivas verificações. O smoke
também remove da instalação final o `sudo NOPASSWD`, o autologin e os serviços
temporários da live.

## Estrutura

- `aura-dl/`: núcleo e CLI do downloader, escritos em Rust.
- `browser/aura-dl-bridge/`: extensão e host de mensagens nativas.
- `packages/`: PKGBUILDs e repositório local usados no build da ISO.
- `iso/`: perfil archiso, script de customização e entradas de boot.

## Decisões técnicas

- **Base:** perfil `releng` do archiso, copiado e adaptado. Evita erros de
  estrutura de um perfil do zero.
- **Pacotes oficiais:** tudo que está em `core`, `extra` ou `multilib` vem dos
  repositórios oficiais. Pacotes próprios usam o repositório local `[aura]`.
- **Credenciais da live:** usuário `aura`, senha `aura`.
- **Instalação:** o Calamares cria a conta normal escolhida pelo usuário,
  exige senha e configura `wheel` sem copiar o `NOPASSWD` da live.
- **Sudo na live:** `NOPASSWD` apenas para o grupo `wheel`, somente na live. A
  regra deve ser removida no sistema instalado.
- **Reprodutibilidade:** a versão e o label da ISO derivam de
  `SOURCE_DATE_EPOCH`. No CI, esse valor vem da data do último commit.

## Limitações conhecidas

- A extensão do navegador ainda não é assinada pelo Mozilla; no Firefox
  release ela precisa ser carregada temporariamente em `about:debugging`.
- O `aura-dl` oferece somente a CLI; não há interface gráfica, fila,
  cancelamento nem histórico.
- O instalador ainda depende de validação completa em hardware físico; o CI
  valida a abertura do Calamares e a configuração, não executa uma instalação
  destrutiva em disco real.
- Aplicativos libadwaita (GTK4) ignoram o tema GTK comum.

## Licença

MIT. Veja `LICENSE`.
