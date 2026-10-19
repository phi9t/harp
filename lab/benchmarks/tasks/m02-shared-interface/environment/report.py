from sizes import parse_size


def render(entries):
    lines = []
    for name, text in entries:
        lines.append(f"{name}: {parse_size(text)} bytes")
    return "\n".join(lines)
