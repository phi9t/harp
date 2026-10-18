# Research articles: a brief survey

Reading notes retained from Harp’s earlier article collection. Coverage refers
to the sections previously marked inspected in that collection, rather than a
new review of every paper or its latest version. Results were not independently
reproduced. Bibliography-only entries and implementation snapshots are omitted.

## Orientation

| Reading | Main idea |
| --- | --- |
| [Harness engineering](https://lilianweng.github.io/posts/2026-07-04-harness/) | Maps improvement targets across prompts, context, workflows, code and weights. |
| [Self-Improvements in Modern Agentic Systems](https://arxiv.org/abs/2607.13104) | Organizes improvement methods by what changes and what supplies feedback. |
| [The first ultraintelligent machine](https://philpapers.org/rec/GOOSCT) | Introduces the intelligence-explosion argument as speculation. |
| [Recursive Self-Improvement](https://www.lesswrong.com/posts/JBadX7rwdcRFzGuju/recursive-self-improvement) | Frames recursive changes to the machinery that performs optimization. |
| [When AI Builds Itself](https://www.anthropic.com/institute/recursive-self-improvement) | Distinguishes assisted AI development from autonomous successor development. |

## Context, prompts and workflow search

| Reading | Main idea |
| --- | --- |
| [Self-Refine](https://arxiv.org/abs/2303.17651) | Iterates feedback and revision within a task. |
| [Agentic Context Engineering](https://iclr.cc/virtual/2026/poster/10008343) | Evolves reusable context rather than only rewriting one answer. |
| [Meta Context Engineering](https://arxiv.org/abs/2601.21557) | Adds a meta-level loop for evolving skills and context. |
| [Promptbreeder](https://arxiv.org/abs/2309.16797) | Evolves task prompts together with mutation prompts. |
| [GEPA](https://arxiv.org/abs/2507.19457) | Uses reflective feedback to evolve prompts across a compound system. |
| [AFlow](https://openreview.net/forum?id=z5uVAKwmjf) | Searches executable agent workflows with Monte Carlo tree search. |
| [Automated Design of Agentic Systems](https://proceedings.iclr.cc/paper_files/paper/2025/file/36b7acf6f6010652b3f2a433774a66fe-Paper-Conference.pdf) | Searches agent programs while retaining prior discoveries. |
| [Recursive Language Models](https://arxiv.org/abs/2512.24601) | Externalizes context and uses recursive model calls over selected portions. |

## Harness and agent evolution

| Reading | Main idea |
| --- | --- |
| [Self-Taught Optimizer](https://arxiv.org/abs/2310.02304) | Studies code that improves a code-improvement procedure. |
| [Darwin Gödel Machine](https://arxiv.org/abs/2505.22954) | Searches branching lineages of modified coding-agent repositories. |
| [Hyperagents](https://arxiv.org/abs/2603.19461) | Makes task-agent and meta-agent programs editable. |
| [Meta-Harness](https://arxiv.org/abs/2603.28052) | Optimizes model harnesses with a fixed base model and proposer. |
| [Self-Harness](https://arxiv.org/abs/2606.09498) | Tests harness changes under a fixed-model evaluation protocol. |
| [Agentic Harness Engineering](https://arxiv.org/abs/2604.25850) | Uses execution observations to propose and evaluate harness changes. |
| [DemoEvolve](https://arxiv.org/abs/2605.24539) | Uses demonstrations to supplement sparse harness-evolution feedback. |
| [Harness Updating Is Not Harness Benefit](https://arxiv.org/abs/2605.30621) | Separates producing updates from benefiting from those updates. |

## Program discovery and learning

| Reading | Main idea |
| --- | --- |
| [Quality Diversity](https://www.frontiersin.org/journals/robotics-and-ai/articles/10.3389/frobt.2016.00040/full) | Explains search that preserves diverse useful candidates. |
| [FunSearch](https://www.nature.com/articles/s41586-023-06924-6) | Combines generated programs with an executable evaluator. |
| [AlphaEvolve](https://arxiv.org/abs/2506.13131) | Evolves programs against problem-specific automated evaluation. |
| [ShinkaEvolve](https://arxiv.org/abs/2509.19349) | Studies sample-efficient program evolution with an island archive. |
| [ThetaEvolve](https://arxiv.org/abs/2511.23473) | Combines program discovery with optional reward-shaped weight training. |
| [Learning to Discover at Test Time](https://arxiv.org/abs/2601.16175) | Uses test-time weight training and archive reuse for discovery. |
| [Epistemic Uncertainty for Test-Time Discovery](https://arxiv.org/abs/2605.11328) | Uses uncertainty-guided ensembles during test-time discovery. |
| [Self Improving AI with Harness & Weight Updates](https://arxiv.org/abs/2605.27276) | Explores joint harness and model-weight changes. |
| [Continual Harness](https://arxiv.org/abs/2605.09998) | Studies ongoing harness refinement together with policy learning. |
| [Anchored Self-Play for Code Repair](https://openreview.net/forum?id=lTbBFAoPSA) | Pairs bug generation and repair in anchored self-play. |
| [Absolute Zero](https://arxiv.org/abs/2505.03335) | Uses verifiable code-reasoning self-play to train a solver. |
| [Self-Rewarding Language Models](https://arxiv.org/abs/2401.10020) | Iterates self-instruction, self-judging and preference optimization. |
| [Self-Play Fine-Tuning](https://arxiv.org/abs/2401.01335) | Improves a model through repeated self-play fine-tuning. |
| [LADDER](https://arxiv.org/abs/2503.00735v3) | Combines recursive problem decomposition with verifier-guided training. |
| [Test-Time Reinforcement Learning](https://arxiv.org/abs/2504.16084v3) | Uses majority-vote pseudo-labels as test-time training rewards. |
| [Symbolic Recursive Self-Alignment](https://arxiv.org/abs/2603.21558v1) | Studies verified reasoning data in recursive arithmetic self-training. |

## Research automation and evaluation

| Reading | Main idea |
| --- | --- |
| [The AI Scientist](https://www.nature.com/articles/s41586-026-10265-5) | Studies an end-to-end workflow for automating portions of research. |
| [ScientistOne](https://arxiv.org/abs/2605.26340) | Makes a chain of evidence part of autonomous research evaluation. |
| [Autodata](https://arxiv.org/abs/2606.25996) | Uses an agentic workflow to construct synthetic training data. |
| [Why LLMs Aren’t Scientists Yet](https://arxiv.org/abs/2601.03315) | Reports failure modes and human interventions in four research attempts. |
| [Early science acceleration experiments with GPT-5](https://arxiv.org/abs/2511.16072) | Describes expert-assisted scientific case studies. |
| [PaperBench](https://proceedings.mlr.press/v267/starace25a.html) | Evaluates research-paper replication using detailed rubrics. |
| [RE-Bench](https://proceedings.mlr.press/v267/wijk25a.html) | Compares agents and human experts under explicit time budgets. |
| [MLE-bench](https://arxiv.org/abs/2410.07095) | Evaluates machine-learning engineering on offline competition tasks. |
| [ScienceAgentBench](https://arxiv.org/abs/2410.05080) | Evaluates scientific programming against task-specific requirements. |
| [CORE-Bench](https://arxiv.org/abs/2409.11363) | Measures computational reproducibility of published research. |
| [KernelBench](https://arxiv.org/abs/2502.10517) | Requires GPU-kernel correctness alongside measured performance. |
| [EnvHarness](https://arxiv.org/abs/2608.19880) | Studies environment-side support for agent skill learning and training. |

## Evaluation limits and safeguards

| Reading | Main idea |
| --- | --- |
| [Reward Hacking in Reinforcement Learning](https://lilianweng.github.io/posts/2024-11-28-reward-hacking/) | Surveys reward exploitation and evaluator bias. |
| [Concrete Problems in AI Safety](https://arxiv.org/abs/1606.06565) | Frames reward hacking and supervision as practical research problems. |
| [SAHOO](https://arxiv.org/abs/2603.06333v1) | Proposes drift monitoring and safeguards around higher-order optimization. |
| [AI models collapse when trained on recursively generated data](https://www.nature.com/articles/s41586-024-07566-y) | Studies degradation when generated data recursively replaces training data. |
| [Information-Theoretic Limits of Safety Verification](https://arxiv.org/abs/2603.28650v1) | Analyzes verification guarantees under explicit modeling assumptions. |

The recurring engineering lesson is to identify what changes, keep evaluation
separate from proposal generation, and measure benefits on held-out tasks with
explicit budgets. Better task performance, persistent memory and the ability to
edit an agent do not by themselves establish recursive improvement.
