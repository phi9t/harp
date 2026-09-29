import re


def slugify(value: str) -> str:
    value = value.strip().lower()
    value = re.sub(r"\s+", "-", value)
    return re.sub(r"[^a-z0-9-]", "", value)
