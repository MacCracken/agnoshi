# Benchmarks

Latest: **2026-04-06T03:15:44Z** — commit `5a662b0`

Tracking: `a98eef8` (baseline) → `4dd7668` (optimized) → `5a662b0` (current)

## interpreter_parse_simple

| Benchmark | Baseline (`a98eef8`) | Mid (`4dd7668`) | Current (`5a662b0`) |
|-----------|------|------|------|
| `interpreter_parse_simple` | 140580.0 ns | 142140.0 ns | 218280.0 ns +55% |

## interpreter_parse_list_files

| Benchmark | Baseline (`a98eef8`) | Mid (`4dd7668`) | Current (`5a662b0`) |
|-----------|------|------|------|
| `interpreter_parse_list_files` | 23473.0 ns | 21044.0 ns **-10%** | 32043.0 ns +37% |

## interpreter_parse_cd

| Benchmark | Baseline (`a98eef8`) | Mid (`4dd7668`) | Current (`5a662b0`) |
|-----------|------|------|------|
| `interpreter_parse_cd` | 13344.0 ns | 12247.0 ns **-8%** | 18556.0 ns +39% |

## interpreter_translate_list_files

| Benchmark | Baseline (`a98eef8`) | Mid (`4dd7668`) | Current (`5a662b0`) |
|-----------|------|------|------|
| `interpreter_translate_list_files` | 139.8 ns | 143.9 ns | 166.9 ns +19% |

## interpreter_translate_cd

| Benchmark | Baseline (`a98eef8`) | Mid (`4dd7668`) | Current (`5a662b0`) |
|-----------|------|------|------|
| `interpreter_translate_cd` | 54.91 ns | 57.93 ns +6% | 77.21 ns +41% |

## interpreter_explain_ls

| Benchmark | Baseline (`a98eef8`) | Mid (`4dd7668`) | Current (`5a662b0`) |
|-----------|------|------|------|
| `interpreter_explain_ls` | 15.66 ns | 22.91 ns +46% | 55.81 ns +256% |

## interpreter_explain_cat

| Benchmark | Baseline (`a98eef8`) | Mid (`4dd7668`) | Current (`5a662b0`) |
|-----------|------|------|------|
| `interpreter_explain_cat` | 25.21 ns | 31.07 ns +23% | 71.30 ns +183% |

## interpreter_explain_rm

| Benchmark | Baseline (`a98eef8`) | Mid (`4dd7668`) | Current (`5a662b0`) |
|-----------|------|------|------|
| `interpreter_explain_rm` | 34.14 ns | 36.08 ns +6% | 76.78 ns +125% |

## interpreter_parse_10_commands

| Benchmark | Baseline (`a98eef8`) | Mid (`4dd7668`) | Current (`5a662b0`) |
|-----------|------|------|------|
| `interpreter_parse_10_commands` | 125170.0 ns | 123590.0 ns | 217470.0 ns +74% |

## intent_parsing

| Benchmark | Baseline (`a98eef8`) | Mid (`4dd7668`) | Current (`5a662b0`) |
|-----------|------|------|------|
| `individual/show me all files` | 19454.0 ns | 18882.0 ns | 34264.0 ns +76% |
| `individual/find files named foo` | 31083.0 ns | 14008.0 ns **-55%** | 27259.0 ns **-12%** |
| `individual/install package vim` | 28607.0 ns | 16946.0 ns **-41%** | 29127.0 ns |
| `individual/show running processes` | 19110.0 ns | 16406.0 ns **-14%** | 28555.0 ns +49% |
| `individual/mute track vocals` | 20025.0 ns | 16420.0 ns **-18%** | 29606.0 ns +48% |
| `individual/list edge nodes` | 1804.3 ns | 1425.3 ns **-21%** | 1922.1 ns +7% |
| `individual/shruti create session test` | 21414.0 ns | 19289.0 ns **-10%** | 30856.0 ns +44% |
| `individual/search marketplace for agnostic` | 19934.0 ns | 18577.0 ns **-7%** | 31177.0 ns +56% |
| `individual/search knowledge base for networking` | 20260.0 ns | 19035.0 ns **-6%** | 30170.0 ns +49% |
| `individual/ark install htop` | 13889.0 ns | 12296.0 ns **-11%** | 16398.0 ns +18% |
| `individual/delta list repos` | 3173.2 ns | 2885.6 ns **-9%** | 3631.6 ns +14% |
| `individual/list tasks` | 3792.4 ns | 3462.3 ns **-9%** | 4361.4 ns +15% |
| `individual/show balance` | 3694.8 ns | 3294.5 ns **-11%** | 4043.5 ns +9% |
| `individual/scan ports on 192.168.1.1` | 11143.0 ns | 10538.0 ns **-5%** | 13598.0 ns +22% |
| `individual/show system information` | 18636.0 ns | 16178.0 ns **-13%** | 25754.0 ns +38% |
| `batch/15` | 222410.0 ns | 207890.0 ns **-7%** | 309960.0 ns +39% |
| `batch/100` | 1536.8 µs | 1382.0 µs **-10%** | 2086.5 µs +36% |
| `batch/500` | 7651.7 µs | 6768.9 µs **-12%** | 10239.0 µs +34% |
| `parse_translate/15` | 238700.0 ns | 214660.0 ns **-10%** | 335230.0 ns +40% |
| `parse_translate/100` | 1575.1 µs | 1450.3 µs **-8%** | 2197.0 µs +39% |
| `parse_translate/500` | 7671.9 µs | 6839.6 µs **-11%** | 10781.0 µs +41% |
| `edge_cases` | 115850.0 ns | 102480.0 ns **-12%** | 166050.0 ns +43% |

## system

| Benchmark | Baseline (`a98eef8`) | Mid (`4dd7668`) | Current (`5a662b0`) |
|-----------|------|------|------|
| `session_lifecycle_create_execute_destroy` | 7466.1 µs | 7212.9 µs **-3%** | 8938.4 µs +20% |
| `session_create` | 7452.6 µs | 7193.0 µs **-3%** | 9207.3 µs +24% |
| `parse_translate_pipeline/10` | 194060.0 ns | 195560.0 ns | 290480.0 ns +50% |
| `parse_translate_pipeline/50` | 957950.0 ns | 904390.0 ns **-6%** | 1402.2 µs +46% |
| `parse_translate_pipeline/100` | 1989.6 µs | 1793.1 µs **-10%** | 2772.6 µs +39% |
| `parse_only_pipeline/10` | 184380.0 ns | 173640.0 ns **-6%** | 283480.0 ns +54% |
| `parse_only_pipeline/50` | 904220.0 ns | 856510.0 ns **-5%** | 1337.5 µs +48% |
| `parse_only_pipeline/100` | 1803.3 µs | 1732.8 µs **-4%** | 2668.9 µs +48% |
| `prompt_render_full_all_modules` | 4327.4 ns | 3931.8 ns **-9%** | 5077.2 ns +17% |
| `prompt_render_minimal` | 265.5 ns | 198.7 ns **-25%** | 249.4 ns **-6%** |
| `prompt_render_right_prompt` | 472.7 ns | 398.0 ns **-16%** | 508.9 ns +8% |
| `prompt_render_repeated/10` | 52148.0 ns | 46349.0 ns **-11%** | 59822.0 ns +15% |
| `prompt_render_repeated/50` | 253300.0 ns | 234220.0 ns **-8%** | 296250.0 ns +17% |
| `prompt_render_repeated/100` | 554960.0 ns | 468510.0 ns **-16%** | 600340.0 ns +8% |
| `intent_classification/10` | 236600.0 ns | 179900.0 ns **-24%** | 273000.0 ns +15% |
| `intent_classification/50` | 977420.0 ns | 848020.0 ns **-13%** | 1322.1 µs +35% |
| `intent_classification/100` | 1936.7 µs | 1743.8 µs **-10%** | 2574.5 µs +33% |
| `intent_classification/500` | 9179.3 µs | 8702.7 µs **-5%** | 13325.0 µs +45% |
| `intent_classification_diverse_15` | 279050.0 ns | 246430.0 ns **-12%** | 379020.0 ns +36% |
| `history_add_then_search/100` | 23558.0 ns | 22872.0 ns | 35770.0 ns +52% |
| `history_add_then_search/500` | 71520.0 ns | 67691.0 ns **-5%** | 99179.0 ns +39% |
| `history_add_then_search/1000` | 125540.0 ns | 121210.0 ns **-3%** | 164090.0 ns +31% |
| `history_add_then_search/5000` | 582120.0 ns | 559790.0 ns **-4%** | 752590.0 ns +29% |
| `history_search_preloaded/100` | 8515.5 ns | 8387.9 ns | 13028.0 ns +53% |
| `history_search_preloaded/1000` | 76760.0 ns | 74542.0 ns | 98053.0 ns +28% |
| `history_search_preloaded/5000` | 361870.0 ns | 352920.0 ns | 463420.0 ns +28% |
| `history_get_recent/100` | 45.49 ns | 47.43 ns +4% | 58.12 ns +28% |
| `history_get_recent/1000` | 45.70 ns | 47.21 ns +3% | 57.31 ns +25% |
| `history_get_recent/5000` | 45.67 ns | 47.79 ns +5% | 59.01 ns +29% |
| `explain_pipeline/5` | 175.7 ns | 188.6 ns +7% | 332.8 ns +89% |
| `explain_pipeline/16` | 653.8 ns | 679.0 ns +4% | 1106.4 ns +69% |
| `explain_pipeline/50` | 2066.1 ns | 2103.8 ns | 3378.6 ns +64% |
| `explain_all_16_commands` | 674.0 ns | 683.1 ns | 1075.0 ns +60% |
| `explain_10_unknown_commands` | 400.2 ns | 418.4 ns +5% | 594.3 ns +49% |

---

Generated by `./scripts/bench-history.sh`. History in `bench-history.csv`.
