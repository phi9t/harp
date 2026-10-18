SMALL = {"of", "the", "a", "an", "and", "in", "on"}


def title_case(text):
    words = text.split()
    out = []
    for i, w in enumerate(words):
        lw = w.lower()
        out.append(lw if (i and lw in SMALL) else lw.capitalize())
    return " ".join(out)
