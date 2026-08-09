"""Error-driven memory system candidate."""

import json
from typing import Any

from reference.text_classification.memory_system import MemorySystem, extract_json_field

PROMPT_TEMPLATE = """Classify the input while avoiding previously observed mistakes.

Common confusion corrections:
{corrections}

Input:
{input}

Return strict JSON:
{{"reasoning": "[brief reasoning]", "final_answer": "[label]"}}
"""


class ErrorDrivenMemory(MemorySystem):
    """Stores corrected mistakes and feeds them back as compact confusion rules."""

    def __init__(self, llm):
        super().__init__(llm)
        self._max_rules = 10
        self._rules: list[dict[str, str]] = []

    def _format_corrections(self) -> str:
        if not self._rules:
            return "No prior mistakes recorded."
        lines = []
        for rule in self._rules[-self._max_rules :]:
            lines.append(
                "If an item resembles this input snippet:\n"
                f"{rule['input']}\n"
                f"avoid predicting '{rule['prediction']}' and prefer '{rule['ground_truth']}'."
            )
        return "\n\n".join(lines)

    def predict(self, input: str) -> tuple[str, dict[str, Any]]:
        corrections = self._format_corrections()
        prompt = PROMPT_TEMPLATE.format(corrections=corrections, input=input)
        response = self.call_llm(prompt)
        answer = extract_json_field(response, "final_answer")
        return answer, {
            "strategy": "error_driven",
            "rules": len(self._rules),
            "full_response": response,
        }

    def learn_from_batch(self, batch_results: list[dict[str, Any]]) -> None:
        for row in batch_results:
            if bool(row.get("was_correct")):
                continue
            self._rules.append(
                {
                    "ground_truth": str(row["ground_truth"]),
                    "input": str(row["input"]),
                    "prediction": str(row["prediction"]),
                }
            )
        if len(self._rules) > self._max_rules:
            self._rules = self._rules[-self._max_rules :]

    def get_context_length(self) -> int:
        return len(self._format_corrections())

    def get_state(self) -> str:
        data = {
            "max_rules": self._max_rules,
            "rules": list(self._rules),
        }
        return json.dumps(data, sort_keys=True, separators=(",", ":"))

    def set_state(self, state: str) -> None:
        data = json.loads(state)
        self._max_rules = int(data.get("max_rules", 10))
        self._rules = [
            {
                "ground_truth": str(rule["ground_truth"]),
                "input": str(rule["input"]),
                "prediction": str(rule["prediction"]),
            }
            for rule in data.get("rules", [])
        ]
