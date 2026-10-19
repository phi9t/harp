"""Read the existing Git-managed Markdown tracker at an exact revision."""
from __future__ import annotations

from pathlib import PurePosixPath
import re


def linked_records(path, data):
    text = data.decode('utf-8')
    if not path.startswith('.scratch/') or '/issues/' not in path:
        return []
    # Tracker dependencies are links under Blocked by; prose elsewhere does not
    # create scheduling edges. Resolve lexically inside the committed tree.
    section = re.search(r'(?m)^## Blocked by\s*\n([\s\S]*?)(?=^## |\Z)', text)
    if not section:
        raise ValueError('tracker issue is missing its Blocked by section')
    return _local_links(path, section.group(1))


def _local_links(path, text):
    result = []
    for target in re.findall(r'\]\(([^)\n]+)\)', text):
        target = target.split('#', 1)[0].strip('<>')
        if re.match(r'[a-zA-Z][a-zA-Z0-9+.-]*:', target):
            raise ValueError('tracker dependencies must be committed local records')
        parts = list(PurePosixPath(path).parent.parts)
        if target.startswith('/') or '\\' in target:
            raise ValueError('tracker dependency escapes the repository')
        for part in PurePosixPath(target).parts:
            if part == '..':
                if not parts:
                    raise ValueError('tracker dependency escapes the repository')
                parts.pop()
            elif part != '.':
                parts.append(part)
        name = '/'.join(parts)
        if not name.startswith('.scratch/') or not name.endswith('.md'):
            raise ValueError('tracker dependency must name a Markdown tracker record')
        result.append(name)
    return result


def parent_records(path, data):
    """Parent intent is context, not a completion dependency."""
    if not path.startswith('.scratch/'):
        return []
    parents = re.findall(r'(?m)^Parent:[ \t]*(.*)$', data.decode('utf-8'))
    return [name for text in parents for name in _local_links(path, text)]


def freeze_records(path, data, read):
    """Freeze parent intent and dependency closure at the caller's Git revision."""
    records = {path: data}
    visiting = set()
    visited = set()

    def visit(name):
        if name in visiting:
            raise ValueError('tracker dependency cycle')
        if name in visited:
            return
        visiting.add(name)
        for parent in parent_records(name, records[name]):
            if parent not in records:
                records[parent] = read(parent)
            visit(parent)
        for dependency in linked_records(name, records[name]):
            if dependency in visiting:
                raise ValueError("tracker dependency cycle")
            if dependency not in records:
                records[dependency] = read(dependency)
            text = records[dependency].decode('utf-8')
            if not re.search(r'(?mi)^Status:\s*done\s*$', text):
                raise ValueError('unfinished tracker dependency: ' + dependency)
            visit(dependency)
        visiting.remove(name)
        visited.add(name)

    visit(path)
    return records
