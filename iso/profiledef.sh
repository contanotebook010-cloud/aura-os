#!/usr/bin/env bash
# shellcheck shell=bash disable=SC2034
# Configuração do perfil archiso do Aura OS.
# Copiado do perfil releng e adaptado. Variáveis são lidas pelo mkarchiso.
# SC2034 (variável não usada) é desativado aqui: as variáveis são consumidas
# pelo mkarchiso, que lê este arquivo com `source`, e não pelo próprio script.

# Data de referência do build. SOURCE_DATE_EPOCH torna a ISO reprodutível:
# o mesmo commit gera o mesmo nome e label em qualquer máquina.
_build_epoch="${SOURCE_DATE_EPOCH:-$(date +%s)}"

iso_name="aura-os"
iso_label="AURA_$(date --date="@${_build_epoch}" +%Y%m)"
iso_publisher="Aura do Sigma"
iso_application="Aura OS Live/Install"
iso_version="$(date --date="@${_build_epoch}" +%Y.%m.%d)"
install_dir="arch"
buildmodes=('iso')
# BIOS legado usa syslinux; UEFI usa systemd-boot. Os dois são necessários
# para o requisito de boot em VMware e QEMU nos dois modos.
bootmodes=('bios.syslinux' 'uefi.systemd-boot')
arch='x86_64'
pacman_conf="pacman.conf"
airootfs_image_type="squashfs"
# xz com filtro BCJ x86 reduz o tamanho da imagem do sistema live.
airootfs_image_tool_options=('-comp' 'xz' '-Xbcj' 'x86' '-b' '1M' '-Xdict-size' '1M')

# Permissões explícitas: /root precisa ser privado (750) e os scripts
# executados dentro do sistema precisam de bit de execução. Cada entrada
# aqui tem que existir no airootfs, senão o mkarchiso aborta o build.
file_permissions=(
  ["/etc/shadow"]="0:0:400"
  ["/root"]="0:0:750"
  ["/root/customize_airootfs.sh"]="0:0:755"
  ["/usr/local/bin/aura-first-boot"]="0:0:755"
  ["/usr/local/bin/aura-dl-host"]="0:0:755"
)
