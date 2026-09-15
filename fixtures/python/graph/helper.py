"""Top-level helper."""


def assist(depth):
    if depth:
        return assist(depth - 1)
    return True
