#!/usr/bin/env python3
"""Valida a configuração YAML do Calamares diretamente na árvore do repositório."""

from __future__ import annotations

from pathlib import Path
import sys
import xml.etree.ElementTree as ET

import yaml

ROOT = Path(__file__).resolve().parent / "aura-calamares-config" / "etc" / "calamares"
REQUIRED_SHOW = {"welcome", "locale", "keyboard", "partition", "users", "summary"}
REQUIRED_EXEC = [
    "partition",
    "mount",
    "unpackfs",
    "machineid",
    "locale",
    "keyboard",
    "localecfg",
    "fstab",
    "initcpiocfg",
    "initcpio",
    "removeuser",
    "users",
    "displaymanager",
    "services-systemd",
    "shellprocess@cleanup-live",
    "grubcfg",
    "bootloader",
    "umount",
]
REQUIRED_CONFIGS = {
    "welcome",
    "locale",
    "keyboard",
    "partition",
    "mount",
    "unpackfs",
    "machineid",
    "users",
    "fstab",
    "initcpiocfg",
    "initcpio",
    "displaymanager",
    "services-systemd",
    "removeuser",
    "grubcfg",
    "bootloader",
    "finished",
    "cleanup-live",
}


def load_yaml(path: Path, failures: list[str]) -> dict:
    try:
        with path.open(encoding="utf-8") as stream:
            value = yaml.safe_load(stream)
    except (OSError, yaml.YAMLError) as error:
        failures.append(f"YAML inválido em {path}: {error}")
        return {}
    if not isinstance(value, dict):
        failures.append(f"a raiz de {path} não é um mapa YAML")
        return {}
    return value


def phase_modules(sequence: list, phase: str) -> list[str]:
    modules: list[str] = []
    for item in sequence:
        if isinstance(item, dict) and isinstance(item.get(phase), list):
            modules.extend(str(module) for module in item[phase])
    return modules


def main() -> int:
    failures: list[str] = []
    settings = load_yaml(ROOT / "settings.conf", failures)
    sequence = settings.get("sequence", [])
    show = phase_modules(sequence, "show")
    execute = phase_modules(sequence, "exec")

    missing_show = sorted(REQUIRED_SHOW - set(show))
    if missing_show:
        failures.append(f"etapas de interface ausentes: {', '.join(missing_show)}")
    if execute != REQUIRED_EXEC:
        failures.append(f"ordem de execução inesperada: {execute!r}")
    if settings.get("branding") != "aura":
        failures.append("branding incorreto")

    for path in sorted((ROOT / "modules").glob("*.conf")):
        load_yaml(path, failures)
    for config in sorted(REQUIRED_CONFIGS):
        path = ROOT / "modules" / f"{config}.conf"
        if not path.is_file():
            failures.append(f"configuração ausente: {path}")

    removeuser_index = execute.index("removeuser") if "removeuser" in execute else -1
    users_index = execute.index("users") if "users" in execute else -1
    if removeuser_index < 0 or users_index < 0 or removeuser_index > users_index:
        failures.append("a conta temporária deve ser removida antes de criar o usuário")

    branding = load_yaml(ROOT / "branding/aura/branding.desc", failures)
    for image_name in ("productIcon", "productLogo", "productWelcome"):
        filename = branding.get("images", {}).get(image_name)
        if not filename:
            failures.append(f"imagem de branding ausente: {image_name}")
            continue
        image_path = ROOT / "branding/aura" / str(filename)
        try:
            ET.parse(image_path)
        except (OSError, ET.ParseError) as error:
            failures.append(f"SVG de branding inválido {image_path}: {error}")

    cleanup = load_yaml(ROOT / "modules/cleanup-live.conf", failures)
    if "/usr/local/bin/aura-install-cleanup" not in cleanup.get("script", []):
        failures.append("script de limpeza da live não configurado")

    if failures:
        print("configuração do Calamares inválida:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("configuração do Calamares válida")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
