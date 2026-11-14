# Aura OS

Distribuição Live/instalável baseada em Arch Linux, com XFCE4 estilizado em Material Design, cursor Bibata, ícones Papirus e o downloader `aura-dl` (em Rust).

## Status

Em desenvolvimento. Esta versão contém o perfil archiso e o pipeline de build. Ainda não estão incluídos:

- Tema Material (`theme/`)
- Projeto `aura-dl` em Rust
- PKGBUILDs de pacotes locais
- Instalador (`installer/`)
- Cursor Bibata (download com checksum SHA-256, a ser adicionado)

## Requisitos para build

Uma das opções abaixo:

- Um host Arch Linux com `archiso` instalado (`sudo pacman -S archiso`).
- O workflow de GitHub Actions deste repositório, que roda em `archlinux:latest`.

## Como gerar a ISO localmente

```bash
cd iso
sudo ./build.sh
```

A ISO sai em `iso/out/`, com o checksum SHA-256 ao lado. O build exige root porque o `mkarchiso` monta sistemas de arquivos.

## Como testar

- **QEMU/KVM (BIOS e UEFI):** `qemu-system-x86_64 -enable-kvm -m 4G -cdrom iso/out/aura-os-*.iso`
- **VMware Workstation/Player:** crie uma VM Linux 64 bits, aponte o CD/DVD para a ISO e ative aceleração 3D.

## Decisões técnicas

- **Base:** perfil `releng` do archiso, copiado e adaptado. Evita erros de estrutura de um perfil do zero.
- **Pacotes:** apenas repositórios oficiais (`core`, `extra`, `multilib`). Pacotes do AUR não entram na ISO, só no pós-instalação.
- **Senha da live:** `aura`, temporária. Deve ser trocada no instalador ou no primeiro boot.
- **Sudo na live:** `NOPASSWD` apenas para o grupo `wheel`, somente na live. A regra deve ser removida no sistema instalado.
- **Reprodutibilidade:** a versão e o label da ISO derivam de `SOURCE_DATE_EPOCH`. No CI, esse valor vem da data do último commit.

## Limitações conhecidas

- Os nomes de pacotes em `iso/packages.x86_64` foram conferidos de memória, não com `pacman -Si` em um host Arch. Confirme antes do primeiro build.
- `polkit-gnome` pode ter sido movido do repositório oficial. Se o build falhar por esse pacote, substitua por `polkit-kde-agent` ou remova.
- Este commit ainda não foi testado com um build real. O primeiro build vai revelar erros de pacote.
- Apps libadwaita (GTK4) ignoram o tema GTK comum. Será documentado junto com o tema.
- LibreWolf fica de fora da ISO por ser do AUR.

## Licença

MIT. Veja `LICENSE`.
