import sys

from .errors import InventoryError


def main(argv=None):
    argv = list(sys.argv[1:] if argv is None else argv)
    if not argv:
        print("usage: inventory.cli <quote|allocate> ...", file=sys.stderr)
        return 2
    try:
        raise InventoryError("NOT_IMPLEMENTED", "CLI commands are not implemented")
    except InventoryError as exc:
        print(str(exc), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
