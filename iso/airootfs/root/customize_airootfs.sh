#!/usr/bin/env bash
# Customização da imagem live do Aura OS. Roda dentro do chroot do airootfs
# durante o build do mkarchiso. Nada aqui deve depender de rede sem verificação.
set -euo pipefail

readonly LIVE_USER="aura"
# Senha temporária da live. Deve ser trocada no instalador ou no primeiro boot.
readonly LIVE_PASSWORD="aura"

# Usuário com shell bash e acesso ao sudo via grupo wheel.
if ! id -u "${LIVE_USER}" >/dev/null 2>&1; then
  useradd -m -G wheel -s /bin/bash "${LIVE_USER}"
fi
echo "${LIVE_USER}:${LIVE_PASSWORD}" | chpasswd

# O LightDM exige que o usuário do autologin pertença ao grupo autologin.
# Sem essa associação, o daemon sobe, mas nenhuma sessão XFCE é aberta.
if ! getent group autologin >/dev/null; then
  groupadd --system autologin
fi
usermod --append --groups autologin "${LIVE_USER}"

# NOPASSWD só na live. Esta regra deve ser removida no sistema instalado.
install -d -m 0750 /etc/sudoers.d
printf '%%wheel ALL=(ALL:ALL) NOPASSWD: ALL\n' > /etc/sudoers.d/10-aura-live
chmod 0440 /etc/sudoers.d/10-aura-live

# O preset é pré-carregado no skel e no home da live. Assim, o primeiro login
# já encontra o painel inferior, o menu Iniciar, os temas e os atalhos. O
# script idempotente aura-first-boot consolida os valores no Xfconf.
install_preset() {
  local target_home="$1"
  local target_owner="$2"
  local target_group="$3"

  install -d -m 0755 "${target_home}/.config" "${target_home}/Desktop"
  cp -a -- /usr/share/aura-os/xfce-preset/. "${target_home}/.config/"
  for shortcut in /usr/share/aura-os/shortcuts/*.desktop; do
    install -m 0644 "${shortcut}" "${target_home}/Desktop/$(basename -- "${shortcut}")"
  done
  chown -R "${target_owner}:${target_group}" "${target_home}/.config" "${target_home}/Desktop"
}

install_preset "/etc/skel" "root" "root"
install_preset "/home/${LIVE_USER}" "${LIVE_USER}" "$(id -gn "${LIVE_USER}")"

# Identidade e timezone básicos. Sem estes arquivos, o systemd-firstboot abre
# o assistente interativo de configuração e impede a live de chegar ao XFCE.
printf 'aura\n' > /etc/hostname
printf 'LANG=C.UTF-8\n' > /etc/locale.conf
ln -sfn /usr/share/zoneinfo/UTC /etc/localtime

# Só os serviços necessários para a live. Serviços desnecessários aumentam
# a superfície de ataque e o tempo de boot.
systemctl enable NetworkManager.service
systemctl enable lightdm.service
systemctl enable bluetooth.service
systemctl enable vmtoolsd.service
systemctl enable aura-session-ready.timer

# Autologin no XFCE para a live. Não é configurado no sistema instalado.
# session-wrapper e user-session são explícitos para evitar a tela de login
# infinita quando a sessão XFCE não está disponível.
install -d -m 0755 /etc/lightdm
cat > /etc/lightdm/lightdm.conf <<'EOF'
[LightDM]
logind-check-graphical=false

[Seat:*]
greeter-session=lightdm-gtk-greeter
user-session=xfce
session-wrapper=/etc/lightdm/Xsession
autologin-user=aura
autologin-user-timeout=0
autologin-session=xfce
EOF

# Falha cedo no build se uma alteração futura remover o grupo de autologin ou
# o wrapper de sessão fornecido pelo pacote LightDM do Arch.
getent group autologin >/dev/null
id -nG "${LIVE_USER}" | grep -qw autologin
test -x /etc/lightdm/Xsession
