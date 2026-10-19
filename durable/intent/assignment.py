"""Bind a coding assignment to frozen intent and its owner-authorized scope."""
import hashlib


def bind(records, policy):
    # Authority comes from committed admission inputs. Workers get an assignment,
    # never approval or landing capabilities; these remain in the owner store.
    return {'records': {path: hashlib.sha256(data).hexdigest()
                        for path, data in sorted(records.items())},
            'allowed_paths': list(policy['allowed_paths'])}
