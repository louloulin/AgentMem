"""
Convert the official LoCoMo dataset (locomo10.json) into the AgentMem
LOCOMO benchmark format (data/<category>/session_xxx.json).

Usage:
  python scripts/convert_locomo.py \
    --input LoCoMo/data/locomo10.json \
    --output data

Notes:
- Category mapping is heuristic:
    1 -> single_hop (fact)
    2 -> temporal
    3 -> open_domain / commonsense
    4 -> multi_hop
    5 -> adversarial (uses adversarial_answer)
- Each QA becomes one session file containing the full conversation
  history (all sessions concatenated) plus a single question.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Dict, List, Any


CATEGORY_MAP: Dict[int, str] = {
    1: "single_hop",
    2: "temporal",
    3: "open_domain",
    4: "multi_hop",
    5: "adversarial",
}


def load_locomo(path: Path) -> List[Dict[str, Any]]:
    with path.open() as f:
        return json.load(f)


def flatten_conversation(sample: Dict[str, Any]) -> List[Dict[str, str]]:
    conv = sample["conversation"]
    # collect session indices based on keys like session_1, session_2
    session_indices = sorted(
        {
            int(k.split("_")[1])
            for k in conv
            if k.startswith("session_") and k[len("session_")].isdigit()
        }
    )

    messages: List[Dict[str, str]] = []
    speaker_a = conv.get("speaker_a", "Speaker A")
    speaker_b = conv.get("speaker_b", "Speaker B")

    for idx in session_indices:
        session_key = f"session_{idx}"
        turns = conv.get(session_key, [])
        for turn in turns:
            speaker = turn.get("speaker", "")
            role = "user" if speaker == speaker_a else "assistant"
            text = turn.get("text", "")
            if not text:
                continue
            messages.append({"role": role, "content": text})

    return messages


def convert_sample(sample: Dict[str, Any], out_dir: Path) -> None:
    messages = flatten_conversation(sample)
    sample_id = sample.get("sample_id", "unknown")

    for idx, qa in enumerate(sample.get("qa", []), start=1):
        cat_idx = qa.get("category", 1)
        category = CATEGORY_MAP.get(cat_idx, "single_hop")
        qa_out_dir = out_dir / category
        qa_out_dir.mkdir(parents=True, exist_ok=True)

        expected_answer_raw = qa.get("answer") or qa.get("adversarial_answer") or ""
        expected_answer = str(expected_answer_raw)
        question_text = str(qa.get("question", ""))
        session_id = f"{sample_id}_{idx:03d}"

        session = {
            "session_id": session_id,
            "timestamp": "",
            "messages": messages,
            "questions": [
                {
                    "question_id": f"q_{idx:03d}",
                    "category": category,
                    "question": question_text,
                    "expected_answer": expected_answer,
                    # Evidence is dialog ids like D1:3; keep as-is for traceability.
                    "session_references": [str(x) for x in qa.get("evidence", [])],
                }
            ],
        }

        out_path = qa_out_dir / f"{session_id}.json"
        out_path.write_text(json.dumps(session, ensure_ascii=False, indent=2))


def main() -> None:
    parser = argparse.ArgumentParser(description="Convert LoCoMo data to AgentMem format.")
    parser.add_argument("--input", type=Path, required=True, help="Path to locomo10.json")
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("data"),
        help="Output base directory (will create category subfolders).",
    )
    args = parser.parse_args()

    samples = load_locomo(args.input)
    for sample in samples:
        convert_sample(sample, args.output)

    print(f"Converted {len(samples)} samples to {args.output}")


if __name__ == "__main__":
    main()
