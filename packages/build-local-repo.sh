#!/usr/bin/env bash
# Monta os pacotes locais do Aura OS e publica um repositório pacman.
#
# Os fontes são empacotados a partir da árvore de trabalho, o que permite
# validar uma alteração local antes de criar uma tag. O build da ISO sempre
# usa a mesma árvore do commit em execução.
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
readonly REPO_ROOT
# O repositório tem um único caminho permitido. Isso evita que uma chamada
# acidental apague um diretório arbitrário durante a preparação da ISO.
readonly LOCAL_REPO="/tmp/aura-os-repo"
WORK_DIR="$(mktemp -d /tmp/aura-os-pkgbuild.XXXXXX)"
readonly WORK_DIR
# O usuário de build precisa atravessar o diretório temporário raiz.
chmod 0755 "${WORK_DIR}"
readonly SOURCE_DATE_EPOCH_VALUE="${SOURCE_DATE_EPOCH:-0}"

cleanup() {
  rm -rf -- "${WORK_DIR}"
}
trap cleanup EXIT

fail() {
  echo "erro: $*" >&2
  exit 1
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || fail "comando necessário não encontrado: $1"
}

read_cargo_version() {
  sed -nE 's/^version = "([^"]+)"$/\1/p' "${REPO_ROOT}/aura-dl/Cargo.toml" | head -n 1
}

read_extension_version() {
  sed -nE 's/^[[:space:]]*"version": "([^"]+)",?$/\1/p' \
    "${REPO_ROOT}/browser/aura-dl-bridge/manifest.json" | head -n 1
}

make_source_archive() {
  local source_dir="$1"
  local prefix="$2"
  local output="$3"
  local staging="${WORK_DIR}/source-${prefix}"

  mkdir -p "${staging}"
  cp -a -- "${source_dir}" "${staging}/${prefix}"
  rm -rf -- "${staging}/${prefix}/target" \
    "${staging}/${prefix}/.git" \
    "${staging}/${prefix}/__pycache__" \
    "${staging}/${prefix}/.pytest_cache"
  install -m 0644 "${REPO_ROOT}/LICENSE" "${staging}/${prefix}/LICENSE"

  tar --sort=name \
    --mtime="@${SOURCE_DATE_EPOCH_VALUE}" \
    --owner=0 --group=0 --numeric-owner \
    -C "${staging}" -cf - "${prefix}" \
    | gzip -n > "${output}"
}

build_package() {
  local package_name="$1"
  local package_source="$2"
  local package_dir="${REPO_ROOT}/packages/${package_name}"
  local build_dir="${WORK_DIR}/build-${package_name}"
  local archive="${WORK_DIR}/${package_source}"

  mkdir -p "${build_dir}"
  cp -- "${package_dir}/PKGBUILD" "${build_dir}/PKGBUILD"
  cp -- "${archive}" "${build_dir}/${package_source}"

  # O fonte é gerado da árvore atual. Substituímos o marcador SKIP pelo
  # SHA-256 real antes de chamar o makepkg.
  local source_checksum
  read -r source_checksum _ < <(sha256sum -- "${archive}")
  sed -i \
    "s/^sha256sums=('SKIP')$/sha256sums=('${source_checksum}')/" \
    "${build_dir}/PKGBUILD"

  if (( EUID == 0 )); then
    local builder_user="nobody"
    local builder_group
    builder_group="$(id -gn "${builder_user}")"
    mkdir -p "${WORK_DIR}/home" "${WORK_DIR}/cargo" "${WORK_DIR}/cache"
    chown -R "${builder_user}:${builder_group}" \
      "${WORK_DIR}/home" "${WORK_DIR}/cargo" "${WORK_DIR}/cache" "${build_dir}"

    (
      cd -- "${build_dir}"
      runuser -u "${builder_user}" -- env \
        HOME="${WORK_DIR}/home" \
        CARGO_HOME="${WORK_DIR}/cargo" \
        XDG_CACHE_HOME="${WORK_DIR}/cache" \
        makepkg --noconfirm --nodeps --cleanbuild --nocheck
    )
  else
    (
      cd -- "${build_dir}"
      makepkg --noconfirm --nodeps --cleanbuild --nocheck
    )
  fi

  local package_file
  package_file="$(find "${build_dir}" -maxdepth 1 -type f -name '*.pkg.tar.*' -print -quit)"
  [[ -n "${package_file}" ]] || fail "makepkg não gerou um pacote para ${package_name}"
  cp -- "${package_file}" "${LOCAL_REPO}/"

  local package_contents
  package_contents="$(bsdtar -tf "${package_file}")"

  shift 2
  local required_path
  for required_path in "$@"; do
    grep -Eq "(^|/)${required_path}$" <<< "${package_contents}" \
      || fail "o pacote ${package_name} não contém ${required_path}"
  done
}

require_command cargo
require_command makepkg
require_command repo-add
require_command bsdtar
require_command sha256sum
if (( EUID == 0 )); then
  require_command runuser
fi

[[ "${SOURCE_DATE_EPOCH_VALUE}" =~ ^[0-9]+$ ]] \
  || fail "SOURCE_DATE_EPOCH precisa ser um inteiro"

AURA_DL_VERSION="$(read_cargo_version)"
readonly AURA_DL_VERSION
BRIDGE_VERSION="$(read_extension_version)"
readonly BRIDGE_VERSION
[[ -n "${AURA_DL_VERSION}" ]] || fail "não consegui ler a versão do aura-dl"
[[ -n "${BRIDGE_VERSION}" ]] || fail "não consegui ler a versão da ponte"

rm -rf -- "${LOCAL_REPO}"
install -d -m 0755 "${LOCAL_REPO}"

make_source_archive \
  "${REPO_ROOT}/aura-dl" \
  "aura-dl-${AURA_DL_VERSION}" \
  "${WORK_DIR}/aura-dl-${AURA_DL_VERSION}.tar.gz"
make_source_archive \
  "${REPO_ROOT}/browser/aura-dl-bridge" \
  "aura-dl-bridge-${BRIDGE_VERSION}" \
  "${WORK_DIR}/aura-dl-bridge-${BRIDGE_VERSION}.tar.gz"

build_package \
  "aura-dl" \
  "aura-dl-${AURA_DL_VERSION}.tar.gz" \
  "usr/bin/aura-dl"
build_package \
  "aura-dl-bridge" \
  "aura-dl-bridge-${BRIDGE_VERSION}.tar.gz" \
  "usr/bin/aura-dl-host" \
  "usr/lib/mozilla/native-messaging-hosts/aura_dl.json" \
  "usr/lib/librewolf/native-messaging-hosts/aura_dl.json" \
  "usr/share/aura-os/aura-dl-bridge/manifest.json"

repo-add "${LOCAL_REPO}/aura.db.tar.gz" "${LOCAL_REPO}"/*.pkg.tar.*
echo "repositório local pronto em ${LOCAL_REPO}"
