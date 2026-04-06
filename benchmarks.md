# Benchmarks

Latest: **2026-04-05T07:34:20Z** — commit `c567038`

Tracking: `a98eef8` (baseline) → `4dd7668` (optimized) → `c567038` (current)

## interpreter_parse_simple

| Benchmark | Baseline (`a98eef8`) | Mid (`4dd7668`) | Current (`c567038`) |
|-----------|------|------|------|
| `interpreter_parse_simple` | 140580.0 ns | 142140.0 ns | 184460.0 ns +31% |

## interpreter_parse_list_files

| Benchmark | Baseline (`a98eef8`) | Mid (`4dd7668`) | Current (`c567038`) |
|-----------|------|------|------|
| `interpreter_parse_list_files` | 23473.0 ns | 21044.0 ns **-10%** | 28039.0 ns +19% |

## interpreter_parse_cd

| Benchmark | Baseline (`a98eef8`) | Mid (`4dd7668`) | Current (`c567038`) |
|-----------|------|------|------|
| `interpreter_parse_cd` | 13344.0 ns | 12247.0 ns **-8%** | 18509.0 ns +39% |

## interpreter_translate_list_files

| Benchmark | Baseline (`a98eef8`) | Mid (`4dd7668`) | Current (`c567038`) |
|-----------|------|------|------|
| `interpreter_translate_list_files` | 139.8 ns | 143.9 ns | 156.3 ns +12% |

## interpreter_translate_cd

| Benchmark | Baseline (`a98eef8`) | Mid (`4dd7668`) | Current (`c567038`) |
|-----------|------|------|------|
| `interpreter_translate_cd` | 54.91 ns | 57.93 ns +6% | 66.68 ns +21% |

## interpreter_explain_ls

| Benchmark | Baseline (`a98eef8`) | Mid (`4dd7668`) | Current (`c567038`) |
|-----------|------|------|------|
| `interpreter_explain_ls` | 15.66 ns | 22.91 ns +46% | 25.83 ns +65% |

## interpreter_explain_cat

| Benchmark | Baseline (`a98eef8`) | Mid (`4dd7668`) | Current (`c567038`) |
|-----------|------|------|------|
| `interpreter_explain_cat` | 25.21 ns | 31.07 ns +23% | 34.85 ns +38% |

## interpreter_explain_rm

| Benchmark | Baseline (`a98eef8`) | Mid (`4dd7668`) | Current (`c567038`) |
|-----------|------|------|------|
| `interpreter_explain_rm` | 34.14 ns | 36.08 ns +6% | 41.71 ns +22% |

## interpreter_parse_10_commands

| Benchmark | Baseline (`a98eef8`) | Mid (`4dd7668`) | Current (`c567038`) |
|-----------|------|------|------|
| `interpreter_parse_10_commands` | 125170.0 ns | 123590.0 ns | 171390.0 ns +37% |

## intent_parsing

| Benchmark | Baseline (`a98eef8`) | Mid (`4dd7668`) | Current (`c567038`) |
|-----------|------|------|------|
| `individual/show me all files` | 19454.0 ns | 18882.0 ns | 29295.0 ns +51% |
| `individual/find files named foo` | 31083.0 ns | 14008.0 ns **-55%** | 16518.0 ns **-47%** |
| `individual/install package vim` | 28607.0 ns | 16946.0 ns **-41%** | 24843.0 ns **-13%** |
| `individual/show running processes` | 19110.0 ns | 16406.0 ns **-14%** | 24291.0 ns +27% |
| `individual/mute track vocals` | 20025.0 ns | 16420.0 ns **-18%** | 24121.0 ns +20% |
| `individual/list edge nodes` | 1804.3 ns | 1425.3 ns **-21%** | 1754.9 ns |
| `individual/shruti create session test` | 21414.0 ns | 19289.0 ns **-10%** | 28241.0 ns +32% |
| `individual/search marketplace for agnostic` | 19934.0 ns | 18577.0 ns **-7%** | 29951.0 ns +50% |
| `individual/search knowledge base for networking` | 20260.0 ns | 19035.0 ns **-6%** | 28880.0 ns +43% |
| `individual/ark install htop` | 13889.0 ns | 12296.0 ns **-11%** | 15252.0 ns +10% |
| `individual/delta list repos` | 3173.2 ns | 2885.6 ns **-9%** | 3295.5 ns +4% |
| `individual/list tasks` | 3792.4 ns | 3462.3 ns **-9%** | 3973.5 ns +5% |
| `individual/show balance` | 3694.8 ns | 3294.5 ns **-11%** | 3684.7 ns |
| `individual/scan ports on 192.168.1.1` | 11143.0 ns | 10538.0 ns **-5%** | 12093.0 ns +9% |
| `individual/show system information` | 18636.0 ns | 16178.0 ns **-13%** | 24168.0 ns +30% |
| `batch/15` | 222410.0 ns | 207890.0 ns **-7%** | 278230.0 ns +25% |
| `batch/100` | 1536.8 µs | 1382.0 µs **-10%** | 1922.3 µs +25% |
| `batch/500` | 7651.7 µs | 6768.9 µs **-12%** | 9102.9 µs +19% |
| `parse_translate/15` | 238700.0 ns | 214660.0 ns **-10%** | 290450.0 ns +22% |
| `parse_translate/100` | 1575.1 µs | 1450.3 µs **-8%** | 2031.2 µs +29% |
| `parse_translate/500` | 7671.9 µs | 6839.6 µs **-11%** | 9801.8 µs +28% |
| `edge_cases` | 115850.0 ns | 102480.0 ns **-12%** | 155830.0 ns +35% |

## system

| Benchmark | Baseline (`a98eef8`) | Mid (`4dd7668`) | Current (`c567038`) |
|-----------|------|------|------|
| `session_lifecycle_create_execute_destroy` | 7466.1 µs | 7212.9 µs **-3%** | 8413.1 µs +13% |
| `session_create` | 7452.6 µs | 7193.0 µs **-3%** | 8561.8 µs +15% |
| `parse_translate_pipeline/10` | 194060.0 ns | 195560.0 ns | 298070.0 ns +54% |
| `parse_translate_pipeline/50` | 957950.0 ns | 904390.0 ns **-6%** | 1366.0 µs +43% |
| `parse_translate_pipeline/100` | 1989.6 µs | 1793.1 µs **-10%** | 2579.5 µs +30% |
| `parse_only_pipeline/10` | 184380.0 ns | 173640.0 ns **-6%** | 266160.0 ns +44% |
| `parse_only_pipeline/50` | 904220.0 ns | 856510.0 ns **-5%** | 1277.9 µs +41% |
| `parse_only_pipeline/100` | 1803.3 µs | 1732.8 µs **-4%** | 2631.1 µs +46% |
| `prompt_render_full_all_modules` | 4327.4 ns | 3931.8 ns **-9%** | 4894.3 ns +13% |
| `prompt_render_minimal` | 265.5 ns | 198.7 ns **-25%** | 253.3 ns **-5%** |
| `prompt_render_right_prompt` | 472.7 ns | 398.0 ns **-16%** | 493.9 ns +4% |
| `prompt_render_repeated/10` | 52148.0 ns | 46349.0 ns **-11%** | 55554.0 ns +7% |
| `prompt_render_repeated/50` | 253300.0 ns | 234220.0 ns **-8%** | 276960.0 ns +9% |
| `prompt_render_repeated/100` | 554960.0 ns | 468510.0 ns **-16%** | 542610.0 ns |
| `intent_classification/10` | 236600.0 ns | 179900.0 ns **-24%** | 268040.0 ns +13% |
| `intent_classification/50` | 977420.0 ns | 848020.0 ns **-13%** | 1305.0 µs +34% |
| `intent_classification/100` | 1936.7 µs | 1743.8 µs **-10%** | 2581.8 µs +33% |
| `intent_classification/500` | 9179.3 µs | 8702.7 µs **-5%** | 12616.0 µs +37% |
| `intent_classification_diverse_15` | 279050.0 ns | 246430.0 ns **-12%** | 370950.0 ns +33% |
| `history_add_then_search/100` | 23558.0 ns | 22872.0 ns | 26493.0 ns +12% |
| `history_add_then_search/500` | 71520.0 ns | 67691.0 ns **-5%** | 77483.0 ns +8% |
| `history_add_then_search/1000` | 125540.0 ns | 121210.0 ns **-3%** | 135640.0 ns +8% |
| `history_add_then_search/5000` | 582120.0 ns | 559790.0 ns **-4%** | 646200.0 ns +11% |
| `history_search_preloaded/100` | 8515.5 ns | 8387.9 ns | 9459.0 ns +11% |
| `history_search_preloaded/1000` | 76760.0 ns | 74542.0 ns | 84754.0 ns +10% |
| `history_search_preloaded/5000` | 361870.0 ns | 352920.0 ns | 397030.0 ns +10% |
| `history_get_recent/100` | 45.49 ns | 47.43 ns +4% | 57.23 ns +26% |
| `history_get_recent/1000` | 45.70 ns | 47.21 ns +3% | 55.87 ns +22% |
| `history_get_recent/5000` | 45.67 ns | 47.79 ns +5% | 56.72 ns +24% |
| `explain_pipeline/5` | 175.7 ns | 188.6 ns +7% | 232.1 ns +32% |
| `explain_pipeline/16` | 653.8 ns | 679.0 ns +4% | 864.5 ns +32% |
| `explain_pipeline/50` | 2066.1 ns | 2103.8 ns | 2553.1 ns +24% |
| `explain_all_16_commands` | 674.0 ns | 683.1 ns | 855.8 ns +27% |
| `explain_10_unknown_commands` | 400.2 ns | 418.4 ns +5% | 516.7 ns +29% |

---

Generated by `./scripts/bench-history.sh`. History in `bench-history.csv`.
