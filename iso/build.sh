#!/usr/bin/env bash
# Wrapper de build do Aura OS. Usado localmente e pelo GitHub Actions.
# Requer root porque o mkarchiso monta sistemas de arquivos.
set -euo pipefail

# Declaração e atribuição separadas: com readonly na mesma linha, um erro
# dentro da substituição de comando seria mascarado (SC2155).
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR
readonly WORK_DIR="${SCRIPT_DIR}/work"
readonly OUT_DIR="${SCRIPT_DIR}/out"

if [[ "${EUID}" -ne 0 ]]; then
  echo "erro: execute como root (sudo ./build.sh)" >&2
  exit 1
fi

if ! command -v mkarchiso >/dev/null 2>&1; then
  echo "erro: mkarchiso não encontrado. Instale com: pacman -S archiso" >&2
  exit 1
fi

# Limpa o diretório de trabalho anterior. Um build parcial deixado para trás
# pode misturar pacotes antigos com novos e gerar uma ISO inconsistente.
rm -rf -- "${WORK_DIR}"
mkdir -p -- "${OUT_DIR}"

mkarchiso -v -w "${WORK_DIR}" -o "${OUT_DIR}" "${SCRIPT_DIR}"

# Gera o checksum SHA-256 ao lado da ISO para verificação de integridade.
shopt -s nullglob
for iso in "${OUT_DIR}"/*.iso; do
  sha256sum -- "${iso}" > "${iso}.sha256"
  echo "ISO gerada: ${iso}"
done
