def parse_size(text):
    number, unit = text.split()
    return int(float(number) * {"B": 1, "KiB": 1024, "MiB": 1024**2, "GiB": 1024**3}[unit])


def format_size(n):
    if n < 1024:
        return f"{n} B"
    value = float(n)
    for unit in ("KiB", "MiB", "GiB"):
        value /= 1024
        if value < 1024 or unit == "GiB":
            return f"{value:.1f} {unit}"
