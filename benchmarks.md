# Benchmarks

Latest: **2026-04-04T22:14:29Z** — commit `4dd7668`

Tracking: `a98eef8` (baseline) → `a98eef8` (optimized) → `4dd7668` (current)

## interpreter_parse_simple

| Benchmark | Baseline (`a98eef8`) | Mid (`a98eef8`) | Current (`4dd7668`) |
|-----------|------|------|------|
| `interpreter_parse_simple` | 140580.0 ns | 153230.0 ns +9% | 146350.0 ns +4% |

## interpreter_parse_list_files

| Benchmark | Baseline (`a98eef8`) | Mid (`a98eef8`) | Current (`4dd7668`) |
|-----------|------|------|------|
| `interpreter_parse_list_files` | 23473.0 ns | 21553.0 ns **-8%** | 22255.0 ns **-5%** |

## interpreter_parse_cd

| Benchmark | Baseline (`a98eef8`) | Mid (`a98eef8`) | Current (`4dd7668`) |
|-----------|------|------|------|
| `interpreter_parse_cd` | 13344.0 ns | 12607.0 ns **-6%** | 12381.0 ns **-7%** |

## interpreter_translate_list_files

| Benchmark | Baseline (`a98eef8`) | Mid (`a98eef8`) | Current (`4dd7668`) |
|-----------|------|------|------|
| `interpreter_translate_list_files` | 139.8 ns | 145.2 ns +4% | 134.5 ns **-4%** |

## interpreter_translate_cd

| Benchmark | Baseline (`a98eef8`) | Mid (`a98eef8`) | Current (`4dd7668`) |
|-----------|------|------|------|
| `interpreter_translate_cd` | 54.91 ns | 54.72 ns | 58.88 ns +7% |

## interpreter_explain_ls

| Benchmark | Baseline (`a98eef8`) | Mid (`a98eef8`) | Current (`4dd7668`) |
|-----------|------|------|------|
| `interpreter_explain_ls` | 15.66 ns | 16.18 ns +3% | 23.63 ns +51% |

## interpreter_explain_cat

| Benchmark | Baseline (`a98eef8`) | Mid (`a98eef8`) | Current (`4dd7668`) |
|-----------|------|------|------|
| `interpreter_explain_cat` | 25.21 ns | 26.23 ns +4% | 33.04 ns +31% |

## interpreter_explain_rm

| Benchmark | Baseline (`a98eef8`) | Mid (`a98eef8`) | Current (`4dd7668`) |
|-----------|------|------|------|
| `interpreter_explain_rm` | 34.14 ns | 34.50 ns | 36.20 ns +6% |

## interpreter_parse_10_commands

| Benchmark | Baseline (`a98eef8`) | Mid (`a98eef8`) | Current (`4dd7668`) |
|-----------|------|------|------|
| `interpreter_parse_10_commands` | 125170.0 ns | 125060.0 ns | 129410.0 ns +3% |

## intent_parsing

| Benchmark | Baseline (`a98eef8`) | Mid (`a98eef8`) | Current (`4dd7668`) |
|-----------|------|------|------|
| `individual/show me all files` | 19454.0 ns | 20968.0 ns +8% | 20828.0 ns +7% |
| `individual/find files named foo` | 31083.0 ns | 23247.0 ns **-25%** | 15776.0 ns **-49%** |
| `individual/install package vim` | 28607.0 ns | 38062.0 ns +33% | 19080.0 ns **-33%** |
| `individual/show running processes` | 19110.0 ns | 26094.0 ns +37% | 17312.0 ns **-9%** |
| `individual/mute track vocals` | 20025.0 ns | 18050.0 ns **-10%** | 18772.0 ns **-6%** |
| `individual/list edge nodes` | 1804.3 ns | 1774.8 ns | 1583.1 ns **-12%** |
| `individual/shruti create session test` | 21414.0 ns | 20251.0 ns **-5%** | 19796.0 ns **-8%** |
| `individual/search marketplace for agnostic` | 19934.0 ns | 26791.0 ns +34% | 19794.0 ns |
| `individual/search knowledge base for networking` | 20260.0 ns | 39370.0 ns +94% | 19907.0 ns |
| `individual/ark install htop` | 13889.0 ns | 13711.0 ns | 12927.0 ns **-7%** |
| `individual/delta list repos` | 3173.2 ns | 3144.1 ns | 3164.8 ns |
| `individual/list tasks` | 3792.4 ns | 3803.6 ns | 3838.7 ns |
| `individual/show balance` | 3694.8 ns | 3617.5 ns | 3704.1 ns |
| `individual/scan ports on 192.168.1.1` | 11143.0 ns | 11254.0 ns | 10955.0 ns |
| `individual/show system information` | 18636.0 ns | 18526.0 ns | 15810.0 ns **-15%** |
| `batch/15` | 222410.0 ns | 233540.0 ns +5% | 210880.0 ns **-5%** |
| `batch/100` | 1536.8 µs | 1463.0 µs **-5%** | 1408.1 µs **-8%** |
| `batch/500` | 7651.7 µs | 7146.9 µs **-7%** | 6941.0 µs **-9%** |
| `parse_translate/15` | 238700.0 ns | 225440.0 ns **-6%** | 227570.0 ns **-5%** |
| `parse_translate/100` | 1575.1 µs | 1536.9 µs | 1481.2 µs **-6%** |
| `parse_translate/500` | 7671.9 µs | 7429.1 µs **-3%** | 7655.2 µs |
| `edge_cases` | 115850.0 ns | 110430.0 ns **-5%** | 108490.0 ns **-6%** |

## system

| Benchmark | Baseline (`a98eef8`) | Mid (`a98eef8`) | Current (`4dd7668`) |
|-----------|------|------|------|
| `session_lifecycle_create_execute_destroy` | 7466.1 µs | 7919.0 µs +6% | 7662.6 µs |
| `session_create` | 7452.6 µs | 7878.1 µs +6% | 7422.7 µs |
| `parse_translate_pipeline/10` | 194060.0 ns | 201700.0 ns +4% | 179950.0 ns **-7%** |
| `parse_translate_pipeline/50` | 957950.0 ns | 983590.0 ns | 925190.0 ns **-3%** |
| `parse_translate_pipeline/100` | 1989.6 µs | 2060.3 µs +4% | 1797.4 µs **-10%** |
| `parse_only_pipeline/10` | 184380.0 ns | 255780.0 ns +39% | 179460.0 ns |
| `parse_only_pipeline/50` | 904220.0 ns | 921380.0 ns | 856230.0 ns **-5%** |
| `parse_only_pipeline/100` | 1803.3 µs | 1817.6 µs | 1722.1 µs **-5%** |
| `prompt_render_full_all_modules` | 4327.4 ns | 4242.3 ns | 4005.8 ns **-7%** |
| `prompt_render_minimal` | 265.5 ns | 254.0 ns **-4%** | 194.3 ns **-27%** |
| `prompt_render_right_prompt` | 472.7 ns | 458.6 ns | 408.4 ns **-14%** |
| `prompt_render_repeated/10` | 52148.0 ns | 48870.0 ns **-6%** | 47545.0 ns **-9%** |
| `prompt_render_repeated/50` | 253300.0 ns | 253920.0 ns | 240610.0 ns **-5%** |
| `prompt_render_repeated/100` | 554960.0 ns | 500400.0 ns **-10%** | 480620.0 ns **-13%** |
| `intent_classification/10` | 236600.0 ns | 190910.0 ns **-19%** | 178050.0 ns **-25%** |
| `intent_classification/50` | 977420.0 ns | 899630.0 ns **-8%** | 875360.0 ns **-10%** |
| `intent_classification/100` | 1936.7 µs | 1753.9 µs **-9%** | 1744.3 µs **-10%** |
| `intent_classification/500` | 9179.3 µs | 9166.8 µs | 8539.5 µs **-7%** |
| `intent_classification_diverse_15` | 279050.0 ns | 251170.0 ns **-10%** | 244930.0 ns **-12%** |
| `history_add_then_search/100` | 23558.0 ns | 23410.0 ns | 23426.0 ns |
| `history_add_then_search/500` | 71520.0 ns | 68915.0 ns **-4%** | 72158.0 ns |
| `history_add_then_search/1000` | 125540.0 ns | 124440.0 ns | 128170.0 ns |
| `history_add_then_search/5000` | 582120.0 ns | 580620.0 ns | 581910.0 ns |
| `history_search_preloaded/100` | 8515.5 ns | 15541.0 ns +83% | 8194.1 ns **-4%** |
| `history_search_preloaded/1000` | 76760.0 ns | 137170.0 ns +79% | 73536.0 ns **-4%** |
| `history_search_preloaded/5000` | 361870.0 ns | 394740.0 ns +9% | 347050.0 ns **-4%** |
| `history_get_recent/100` | 45.49 ns | 55.95 ns +23% | 46.51 ns |
| `history_get_recent/1000` | 45.70 ns | 59.41 ns +30% | 46.64 ns |
| `history_get_recent/5000` | 45.67 ns | 51.13 ns +12% | 46.51 ns |
| `explain_pipeline/5` | 175.7 ns | 171.9 ns | 201.6 ns +15% |
| `explain_pipeline/16` | 653.8 ns | 662.6 ns | 711.2 ns +9% |
| `explain_pipeline/50` | 2066.1 ns | 2889.1 ns +40% | 2233.5 ns +8% |
| `explain_all_16_commands` | 674.0 ns | 1296.3 ns +92% | 696.3 ns +3% |
| `explain_10_unknown_commands` | 400.2 ns | 848.8 ns +112% | 412.3 ns +3% |

---

Generated by `./scripts/bench-history.sh`. History in `bench-history.csv`.
