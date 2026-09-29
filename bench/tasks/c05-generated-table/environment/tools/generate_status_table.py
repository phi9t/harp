#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCHEMA = ROOT / "data" / "status_schema.json"
OUTPUT = ROOT / "src" / "status_table.cpp"


def cxx_string(value: str) -> str:
    return json.dumps(value)


def main() -> None:
    statuses = json.loads(SCHEMA.read_text())
    rows = []
    for status in statuses:
        rows.append(
            "    {"
            + ", ".join(
                [
                    cxx_string(status["code"]),
                    cxx_string(status["label"]),
                    str(int(status["retry_delay_ms"])),
                    "true" if status["terminal"] else "false",
                ]
            )
            + "},"
        )

    OUTPUT.write_text(
        "\n".join(
            [
                '#include "status_table.hpp"',
                "",
                "#include <array>",
                "",
                "namespace {",
                "",
                f"const std::array<StatusInfo, {len(statuses)}> kStatuses{{{{",
                *rows,
                "}};",
                "",
                "}  // namespace",
                "",
                "StatusInfo lookup_status(std::string_view code) {",
                "    for (const auto& status : kStatuses) {",
                "        if (status.code == code) {",
                "            return status;",
                "        }",
                "    }",
                '    return {"unknown", "Unknown", 0, true};',
                "}",
                "",
                "std::vector<std::string> retryable_codes() {",
                "    std::vector<std::string> codes;",
                "    for (const auto& status : kStatuses) {",
                "        if (!status.terminal && status.retry_delay_ms > 0) {",
                "            codes.push_back(status.code);",
                "        }",
                "    }",
                "    return codes;",
                "}",
                "",
            ]
        )
    )


if __name__ == "__main__":
    main()
