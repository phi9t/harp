from sizes import parse_size, format_size


def render(entries):
    lines = []
    for name, text in entries:
        lines.append(f"{name}: {format_size(parse_size(text))}")
    return "\n".join(lines)
