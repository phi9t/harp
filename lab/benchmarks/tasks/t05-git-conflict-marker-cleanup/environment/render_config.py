#!/usr/bin/env python3
import configparser
import sys


def load_config(path: str) -> dict[str, object]:
    parser = configparser.ConfigParser()
    with open(path, "r", encoding="utf-8") as handle:
        parser.read_file(handle)

    mode = parser.get("server", "mode")
    tls = parser.get("server", "tls")
    audit = parser.get("audit", "enabled")
    audit_format = parser.get("audit", "format")
    retries = parser.getint("server", "retries")

    if mode not in {"production", "staging"}:
        raise ValueError(f"unsupported mode: {mode}")
    if mode == "production" and tls != "required":
        raise ValueError("production requires tls=required")
    if audit == "enabled" and audit_format != "json":
        raise ValueError("enabled audit requires format=json")
    if retries < 1:
        raise ValueError("retries must be positive")

    return {
        "host": parser.get("server", "host"),
        "port": parser.getint("server", "port"),
        "mode": mode,
        "tls": tls,
        "audit": audit,
        "audit_format": audit_format,
        "retries": retries,
    }


def render(config: dict[str, object]) -> str:
    scheme = "https" if config["tls"] == "required" else "http"
    audit = config["audit_format"] if config["audit"] == "enabled" else "off"
    return (
        f"url={scheme}://{config['host']}:{config['port']} "
        f"mode={config['mode']} audit={audit} retries={config['retries']}"
    )


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: render_config.py CONFIG", file=sys.stderr)
        return 2
    try:
        print(render(load_config(argv[1])))
    except Exception as exc:
        print(f"config error: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
