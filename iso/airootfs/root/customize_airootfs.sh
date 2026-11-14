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

# NOPASSWD só na live. Esta regra deve ser removida no sistema instalado.
install -d -m 0750 /etc/sudoers.d
printf '%%wheel ALL=(ALL:ALL) NOPASSWD: ALL\n' > /etc/sudoers.d/10-aura-live
chmod 0440 /etc/sudoers.d/10-aura-live

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

# Autologin no XFCE para a live. Não é configurado no sistema instalado.
install -d -m 0755 /etc/lightdm
cat > /etc/lightdm/lightdm.conf <<'EOF'
[Seat:*]
autologin-user=aura
autologin-user-timeout=0
autologin-session=xfce
EOF
