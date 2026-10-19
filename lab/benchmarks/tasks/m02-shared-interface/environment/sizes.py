def parse_size(text):
    number, unit = text.split()
    return int(float(number) * {"B": 1, "KiB": 1024, "MiB": 1024**2, "GiB": 1024**3}[unit])
