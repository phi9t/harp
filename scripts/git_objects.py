"""Batch immutable Git object hashing for provenance verification."""
import hashlib
import subprocess


def blob_digests(root, identities):
    identities=sorted(set(identities))
    output=subprocess.check_output(['git','--no-replace-objects','cat-file','--batch'],cwd=root,
        input=''.join(i+'\n' for i in identities).encode())
    result={};offset=0
    for expected in identities:
        end=output.index(b'\n',offset)
        identity,kind,raw_size=output[offset:end].decode().split()
        if identity!=expected or kind!='blob':raise ValueError('unexpected provenance Git object')
        start=end+1;size=int(raw_size)
        result[identity]=hashlib.sha256(output[start:start+size]).hexdigest()
        offset=start+size+1
    if offset!=len(output):raise ValueError('unexpected Git object trailing bytes')
    return result
