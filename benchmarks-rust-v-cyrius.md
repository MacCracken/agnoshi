# Benchmarks: Rust vs Cyrius

Rust baseline: commit `5a662b0` (2026-04-06), criterion v0.8, `--release` (opt=3, lto=thin, codegen-units=1)
Cyrius: port commit (2026-04-13), cc3 v4.1.1 (opt: constant folding, DSE, inline)

## Codebase Comparison

| Metric | Rust | Cyrius | Change |
|--------|------|--------|--------|
| Source lines | 27,251 | 4,042 | −85% |
| Test lines | ~4,500 | 564 | −87% |
| Binary size | 3.8 MB | TBD | |
| Dependencies | 24 crates | 13 stdlib | −46% |
| Compile time | ~15s (release) | TBD | |

## Rust Baseline (last run: 2026-04-06)

### Intent Parsing — Individual

| Benchmark | Time (ns) | Time |
|-----------|-----------|------|
| parse_simple ("show me all files") | 218,280 | 218.3 us |
| parse_list_files | 32,043 | 32.0 us |
| parse_cd | 18,556 | 18.6 us |
| "show me all files" | 34,264 | 34.3 us |
| "find files named foo" | 27,259 | 27.3 us |
| "install package vim" | 29,127 | 29.1 us |
| "show running processes" | 28,555 | 28.6 us |
| "show system information" | 25,754 | 25.8 us |
| "scan ports on 192.168.1.1" | 13,598 | 13.6 us |
| "ark install htop" | 16,398 | 16.4 us |
| "list edge nodes" | 1,922 | 1.9 us |
| "delta list repos" | 3,632 | 3.6 us |
| "list tasks" | 4,361 | 4.4 us |
| "show balance" | 4,044 | 4.0 us |

### Intent Parsing — Translation

| Benchmark | Time (ns) | Time |
|-----------|-----------|------|
| translate_list_files | 167 | 167 ns |
| translate_cd | 77 | 77 ns |

### Intent Parsing — Batch

| Benchmark | Time (ns) | Time |
|-----------|-----------|------|
| batch/15 | 309,960 | 310.0 us |
| batch/100 | 2,086,500 | 2.09 ms |
| batch/500 | 10,239,000 | 10.24 ms |

### Parse + Translate Pipeline

| Benchmark | Time (ns) | Time |
|-----------|-----------|------|
| parse_translate/15 | 335,230 | 335.2 us |
| parse_translate/100 | 2,197,000 | 2.20 ms |
| parse_translate/500 | 10,781,000 | 10.78 ms |

### System Benchmarks

| Benchmark | Time (ns) | Time |
|-----------|-----------|------|
| session_create | 9,207,300 | 9.21 ms |
| session_lifecycle | 8,938,400 | 8.94 ms |
| parse_translate_pipeline/10 | 290,480 | 290.5 us |
| parse_translate_pipeline/50 | 1,402,200 | 1.40 ms |
| parse_translate_pipeline/100 | 2,772,600 | 2.77 ms |
| prompt_render_full | 5,077 | 5.1 us |
| prompt_render_minimal | 249 | 249 ns |
| prompt_render_right | 509 | 509 ns |
| prompt_render_repeated/10 | 59,822 | 59.8 us |
| prompt_render_repeated/100 | 600,340 | 600.3 us |

### Intent Classification

| Benchmark | Time (ns) | Time |
|-----------|-----------|------|
| classify/10 | 273,000 | 273.0 us |
| classify/50 | 1,322,100 | 1.32 ms |
| classify/100 | 2,574,500 | 2.57 ms |
| classify/500 | 13,325,000 | 13.33 ms |
| classify_diverse_15 | 379,020 | 379.0 us |

### History Operations

| Benchmark | Time (ns) | Time |
|-----------|-----------|------|
| add_then_search/100 | 35,770 | 35.8 us |
| add_then_search/1000 | 164,090 | 164.1 us |
| add_then_search/5000 | 752,590 | 752.6 us |
| search_preloaded/100 | 13,028 | 13.0 us |
| search_preloaded/1000 | 98,053 | 98.1 us |
| search_preloaded/5000 | 463,420 | 463.4 us |
| get_recent/1000 | 57 | 57 ns |
| get_recent/5000 | 59 | 59 ns |

### Explain Pipeline

| Benchmark | Time (ns) | Time |
|-----------|-----------|------|
| explain/5 | 333 | 333 ns |
| explain/16 | 1,106 | 1.1 us |
| explain/50 | 3,379 | 3.4 us |
| explain_all_16 | 1,075 | 1.1 us |

## Cyrius Results

**Status:** Benchmark suite compiles and runs (`tests/bench_core.bcyr`). Runtime segfaults due to string type mismatch — cstring literals (`"hello"`) passed to `lib/str.cyr` functions (`str_len`, `str_trim`, `str_sub`) that expect Str fat pointers. Three type systems in play:
- **cstring** — null-terminated byte ptr (string literals, lib/string.cyr operations: strlen, streq, memcpy)
- **Str** — fat pointer `{data, len}` (lib/str.cyr operations: str_len, str_trim, str_sub)
- **Sanitize helpers** — switched to cstring semantics (strlen-based)

Interpreter mixes both types; needs a pass to pick one convention throughout. Once aligned, bench will run.

**Build progress:**
- Struct construction: alloc+store64 pattern throughout (21 files rewritten)
- API: str_byte_at added, str_builder_add/build, str_print, str_from_int
- Match: split into sub-functions to stay under cc3 per-function limits
- Includes: translate.cyr before interpreter.cyr (forward decl resolution)
- match variable -> ok_flag (match is reserved word)

Results will be appended to `bench-history.csv` and this document updated with:
- Per-benchmark comparison (Rust ns vs Cyrius ns)
- Delta percentage
- Analysis of where Cyrius is faster (no regex overhead, no allocation) vs slower (no SIMD, interpreted string ops)

### Expected Performance Characteristics

| Factor | Rust Advantage | Cyrius Advantage |
|--------|---------------|-----------------|
| Regex compilation | 150+ compiled regexes (LazyLock) | No regex — keyword matching, no compile cost |
| String allocation | Heap allocation per match | Bump allocator, no free overhead |
| Translation dispatch | Enum match + trait objects | Direct integer match, no vtable |
| Binary startup | 3.8MB load + dynamic linking | ~85KB static ELF, microsecond startup |
| History search | VecDeque iterator | Linear vec scan (same complexity) |
| Permission classify | HashMap lookup + regex | Direct string comparison chain |

### Notes

- Rust uses compiled regex (150+ patterns) — O(1) per pattern after LazyLock init, but init is ~200us
- Cyrius uses keyword-based `str_contains_ci()` — O(n*m) per check but no init cost
- For interactive shell use (1 parse per keystroke), both are well under perceptible latency (<1ms)
- Batch throughput (500+ commands/sec) matters less than single-command latency for a shell
