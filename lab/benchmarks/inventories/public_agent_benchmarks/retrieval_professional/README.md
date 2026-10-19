# Retrieval and Professional Reasoning Inventories

This tracked directory contains public task inventories for retrieval-heavy and
professional reasoning benchmarks: Finance Agent v2, GDP.pdf, and Harvey LAB.
Each inventory is emitted as CSV and JSONL. These are catalog snapshots, not
runnable Corenius benchmark tasks.

## Regenerate

From the repository root:

```bash
python3 insula/temp_root.py -- python3 lab/benchmarks/inventories/public_agent_benchmarks/scripts/build_retrieval_inventory.py
```

The script is stdlib-only. It writes generated artifacts back to this directory,
uses `.scratch/public_agent_benchmarks/cache/` for local clones and fetched HTML,
uses the HuggingFace dataset-server API for the GDP.pdf public mirror, and reads
the Artificial Analysis GDP.pdf leaderboard page for official displayed
examples.

## Counts

| Inventory | Rows | Source scope |
|---|---:|---|
| `finance_agent_v2_public_questions` | 27 | `vals-ai/finance-agent-v2`, `data/public.csv` |
| `gdp_pdf_official_public_samples` | 2 | `surge-ai/gdp-pdf`, `task_packs/sample_pdf/tasks/*.yaml` |
| `gdp_pdf_public_mirror_tasks` | 100 | Public HuggingFace mirror `cloudybean/GDP.pdf`, `test` split |
| `gdp_pdf_artificial_analysis_examples` | 5 | Official Artificial Analysis GDP.pdf leaderboard examples |
| `harvey_lab_public_tasks` | 2010 | `harveyai/harvey-labs`, every `tasks/**/task.json` |

The same numbers are recorded in `counts.json`.

## Files

| File stem | Format | What is included |
|---|---|---|
| `finance_agent_v2_public_questions` | CSV, JSONL | Question text, question type, expert time, rubric JSON, rubric count, must-pass count, severity sum, source commit |
| `gdp_pdf_official_public_samples` | CSV, JSONL | Official harness sample id, prompt, attachments, rubric JSON, metadata, source commit |
| `gdp_pdf_public_mirror_tasks` | CSV, JSONL | Task id, prompt, domain, PDF path, rubric JSON, rubric count, mirror dataset URL |
| `gdp_pdf_artificial_analysis_examples` | CSV, JSONL | Public example id, title, domain, prompt, PDF URL, example model, all-pass/mean-pass result, criterion-level judge rationales |
| `harvey_lab_public_tasks` | CSV, JSONL | Task id, practice area, title, work type, tags, instructions, deliverables, criterion count, document count, document extensions, source commit |

Some CSV fields contain embedded newlines because task instructions and rubrics
are long. Use a CSV parser, not line counts, to count CSV records.

## Source Notes

- Finance Agent v2 has 927 expert-reviewed questions across public, private
  validation, and held-out test splits according to the public methodology page.
  This inventory includes the 27 public rows shipped in the GitHub repository.
  The private validation and test splits are not public task inventories.
- Finance Agent v2's public set is balanced across the nine taxonomy categories:
  3 questions each for Adjustments, Comparables, Disclosure Analysis, Earnings
  Analysis, Financial Modeling, General Qualitative Analysis, General
  Quantitative Analysis, Market Analysis, and Precedents.
- GDP.pdf's official Artificial Analysis page says the benchmark has 100 tasks
  across ten domains, 4,592 source pages, and 1,275 atomic grading criteria. It
  links to `surgeai/GDP.pdf`, but unauthenticated HuggingFace row access returned
  401 in this run. The official `surge-ai/gdp-pdf` GitHub harness repo contains
  two public sample tasks. The 100-row inventory here uses a public mirror
  (`cloudybean/GDP.pdf`) with the same row count and schema. The five Artificial
  Analysis examples are kept in a separate first-party examples file.
- Harvey LAB's public GitHub repository currently contains 2,010 public
  `task.json` tasks. Artificial Analysis also describes a private Harvey LAB
  evaluation with 120 tasks; that private AA evaluation is not the same thing as
  the public Harvey repository inventory here.

## Distribution Snapshots

Finance Agent v2 public question types:

| Question type | Rows |
|---|---:|
| Adjustments | 3 |
| Comparables | 3 |
| Disclosure Analysis | 3 |
| Earnings Analysis | 3 |
| Financial Modeling | 3 |
| General Qualitative Analysis | 3 |
| General Quantitative Analysis | 3 |
| Market Analysis | 3 |
| Precedents | 3 |

GDP.pdf public mirror domains:

| Domain | Rows |
|---|---:|
| Construction | 10 |
| Engineering | 10 |
| Finance/Investing | 9 |
| HR | 9 |
| Healthcare | 11 |
| Insurance | 9 |
| Legal | 7 |
| Manufacturing/Supply Chains | 10 |
| Real Estate | 10 |
| STEM/Research | 15 |

Harvey LAB public practice areas:

| Practice area | Rows |
|---|---:|
| contracts | 498 |
| firm-knowledge | 250 |
| corporate-ma | 161 |
| intellectual-property | 147 |
| corporate-governance | 97 |
| trusts-estates-private-client | 77 |
| funds-asset-management | 66 |
| litigation-dispute-resolution | 52 |
| data-privacy-cybersecurity | 44 |
| environmental-esg | 44 |
| real-estate | 44 |
| emerging-companies-venture-capital | 43 |
| healthcare-life-sciences | 43 |
| international-trade-sanctions | 41 |
| employment-labor | 39 |
| arbitration-international-dispute-resolution | 37 |
| banking-finance | 37 |
| bankruptcy-restructuring | 36 |
| capital-markets | 35 |
| tax | 34 |
| antitrust-competition | 33 |
| energy-natural-resources | 31 |
| insurance | 31 |
| structured-finance-securitization | 31 |
| immigration | 27 |
| white-collar-defense-investigations | 21 |
| diligence | 11 |
