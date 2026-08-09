"""Recency-window memory system candidate."""

import json
from typing import Any

from reference.text_classification.memory_system import MemorySystem, extract_json_field

PROMPT_TEMPLATE = """Classify the input using the recent labeled examples as guidance.

Recent examples:
{examples}

Input:
{input}

Return strict JSON:
{{"reasoning": "[brief reasoning]", "final_answer": "[label]"}}
"""


class RecencyWindowMemory(MemorySystem):
    """Stores a fixed-size rolling window of recent labeled examples."""

    def __init__(self, llm):
        super().__init__(llm)
        self._window_size = 12
        self._examples: list[dict[str, str]] = []

    def _format_examples(self) -> str:
        if not self._examples:
            return "No prior examples."
        lines = []
        for example in self._examples[-self._window_size :]:
            lines.append(
                f"Input: {example['input']}\nLabel: {example['ground_truth']}"
            )
        return "\n\n".join(lines)

    def predict(self, input: str) -> tuple[str, dict[str, Any]]:
        examples = self._format_examples()
        prompt = PROMPT_TEMPLATE.format(examples=examples, input=input)
        response = self.call_llm(prompt)
        answer = extract_json_field(response, "final_answer")
        return answer, {
            "strategy": "recency_window",
            "window_size": self._window_size,
            "stored_examples": len(self._examples),
            "full_response": response,
        }

    def learn_from_batch(self, batch_results: list[dict[str, Any]]) -> None:
        for row in batch_results:
            self._examples.append(
                {
                    "input": str(row["input"]),
                    "ground_truth": str(row["ground_truth"]),
                }
            )
        if len(self._examples) > self._window_size:
            self._examples = self._examples[-self._window_size :]

    def get_context_length(self) -> int:
        return len(self._format_examples())

    def get_state(self) -> str:
        data = {
            "examples": list(self._examples),
            "window_size": self._window_size,
        }
        return json.dumps(data, sort_keys=True, separators=(",", ":"))

    def set_state(self, state: str) -> None:
        data = json.loads(state)
        self._window_size = int(data.get("window_size", 12))
        self._examples = [
            {
                "input": str(example["input"]),
                "ground_truth": str(example["ground_truth"]),
            }
            for example in data.get("examples", [])
        ]
