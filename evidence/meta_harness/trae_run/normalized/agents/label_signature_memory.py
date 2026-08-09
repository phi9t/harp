"""Label-signature memory system candidate."""

import json
import re
from collections import Counter
from typing import Any

from reference.text_classification.memory_system import MemorySystem, extract_json_field

PROMPT_TEMPLATE = """Classify the input using the observed label-language signatures.

Known labels and signature tokens:
{summary}

Input:
{input}

Return strict JSON:
{{"reasoning": "[brief reasoning]", "final_answer": "[label]"}}
"""

TOKEN_RE = re.compile(r"[A-Za-z0-9']+")


def _tokenize(text: str) -> list[str]:
    return [token.lower() for token in TOKEN_RE.findall(text)]


class LabelSignatureMemory(MemorySystem):
    """Tracks per-label token frequencies and injects a compact label summary."""

    def __init__(self, llm):
        super().__init__(llm)
        self._label_token_counts: dict[str, Counter[str]] = {}
        self._label_example_counts: dict[str, int] = {}
        self._top_k = 6

    def _build_summary(self) -> str:
        if not self._label_token_counts:
            return "No prior examples."
        parts = []
        for label in sorted(self._label_token_counts):
            counts = self._label_token_counts[label]
            top_tokens = [token for token, _ in counts.most_common(self._top_k)]
            token_text = ", ".join(top_tokens) if top_tokens else "(no tokens)"
            examples = self._label_example_counts.get(label, 0)
            parts.append(f"- {label}: {examples} examples; tokens: {token_text}")
        return "\n".join(parts)

    def predict(self, input: str) -> tuple[str, dict[str, Any]]:
        summary = self._build_summary()
        prompt = PROMPT_TEMPLATE.format(summary=summary, input=input)
        response = self.call_llm(prompt)
        answer = extract_json_field(response, "final_answer")
        return answer, {
            "strategy": "label_signature",
            "known_labels": sorted(self._label_token_counts),
            "summary": summary,
            "full_response": response,
        }

    def learn_from_batch(self, batch_results: list[dict[str, Any]]) -> None:
        for row in batch_results:
            label = str(row["ground_truth"])
            if label not in self._label_token_counts:
                self._label_token_counts[label] = Counter()
                self._label_example_counts[label] = 0
            self._label_example_counts[label] += 1
            self._label_token_counts[label].update(_tokenize(str(row["input"])))

    def get_state(self) -> str:
        data = {
            "label_example_counts": {
                label: self._label_example_counts[label]
                for label in sorted(self._label_example_counts)
            },
            "label_token_counts": {
                label: {
                    token: count
                    for token, count in sorted(self._label_token_counts[label].items())
                }
                for label in sorted(self._label_token_counts)
            },
            "top_k": self._top_k,
        }
        return json.dumps(data, sort_keys=True, separators=(",", ":"))

    def set_state(self, state: str) -> None:
        data = json.loads(state)
        self._top_k = int(data.get("top_k", 6))
        self._label_example_counts = {
            str(label): int(count)
            for label, count in data.get("label_example_counts", {}).items()
        }
        self._label_token_counts = {
            str(label): Counter(
                {
                    str(token): int(count)
                    for token, count in token_counts.items()
                }
            )
            for label, token_counts in data.get("label_token_counts", {}).items()
        }
