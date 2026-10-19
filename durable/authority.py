"""Keep a registered owner capability out of subsequent execution projections."""
from pathlib import Path
import sqlite3


def protect_owner(state, roots, db=None):
    """Inspect existing authority only; preparation must not create a database."""
    connection = None
    try:
        if db is None:
            path = Path(state) / "runtime.sqlite"
            if path.is_symlink():
                raise ValueError("owner authority database must not be symlinked")
            if not path.exists():
                return
            connection = sqlite3.connect(path.as_uri() + "?mode=ro", uri=True)
            db = connection
        if not db.execute("SELECT 1 FROM sqlite_master WHERE type='table' AND name='harp_owner'").fetchone():
            return
        row = db.execute("SELECT path FROM harp_owner WHERE id=1").fetchone()
        if row is None:
            return
        owner = Path(row[0]).resolve()
        if any(owner.is_relative_to(Path(root).resolve()) for root in roots):
            raise ValueError("registered owner capability would enter a worker projection; select separate inputs")
    finally:
        if connection is not None:
            connection.close()


def execution_roots(execution):
    config = execution.config
    roots = [config[key] for key in ("code", "state", "inputs_root", "capsule_store") if key in config]
    if "native_manifest" in config:
        roots.append(config["native_manifest"]["rootfs"])
    for layer in config["capsule"]["layers"]:
        if "source" in layer:
            roots.append(layer["source"])
        roots.extend(layer.get("paths", []))
        roots.extend(binding["src"] for binding in layer.get("binds", []))
    return roots
