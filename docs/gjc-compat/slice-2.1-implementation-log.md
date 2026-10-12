# Slice 2.1 Implementation Log: GJC Tool Catalog Semantic Parity

**Issue**: #3323  
**Branch**: `lane/gjc-s21`  
**Target**: Bring read/edit/search/find tools to GJC semantics  
**Status**: In Progress

---

## Completed Work

### Phase 1a: Read Tool Selector Parser ✅
**Commit**: `6b8bdb2c`  
**Date**: 2026-10-13

**Components**:
- ✅ New module: `rust/crates/runtime/src/line_range_selector.rs` (343 LOC)
- ✅ Line range selector parsing (`:L`, `:L-L+N` syntax)
- ✅ Special modes (`:raw`, `:conflicts`)
- ✅ Range merging logic for multi-range selectors
- ✅ 13 unit tests, all passing

**Details**:
- Supports formats: `:50`, `:50-200`, `:50+150`, `:5-16,960-973`
- Automatically merges overlapping/adjacent ranges
- Type: `LineRangeSelector` enum with variants: `Raw`, `Conflicts`, `Lines(Vec<LineRange>)`, `None`

---

## In-Progress Work

### Phase 1b: Integration into ReadFileOutput (🚧)
**Target**: Next commit  
**Scope**:
- [ ] Update `ReadFileInput` struct in `rust/crates/tools/src/lib.rs` to include selector field
- [ ] Update `read_file()` function signature to accept selector parameter
- [ ] Implement selector application in `read_file_in_workspace()`
- [ ] Update output format to include truncation footer
- [ ] Add tests for selector integration

**Estimated LOC**: 600-800  
**Complexity**: Medium

---

## Remaining Work (Planned)

### Phase 1c: Truncation Footers & Metadata
**Target**: After 1b  
**Scope**:
- [ ] Add `totalLines` field to output
- [ ] Add `totalBytes` field to output
- [ ] Generate truncation footer with recovery hints
- [ ] Format footer: `[Showing lines X-Y of Z; re-read path:1-Z for full file]`
- [ ] Add tests for footer generation

**Estimated LOC**: 300-400

### Phase 1d: Structural Summarization (Optional - Lower Priority)
**Target**: Later  
**Scope**:
- [ ] Collapse function bodies to `..` for code files
- [ ] Preserve full lines for prose
- [ ] Add hash line anchors for structural boundaries
- [ ] Support code language detection

**Estimated LOC**: 400-600  
**Complexity**: High

### Batch 2: EDIT Tool Enhancements
**Target**: After Batch 1 complete  
**Scope**:
- [ ] Add patch metadata to output (line numbers, context)
- [ ] Ensure exact match semantics
- [ ] Add comprehensive tests

**Estimated LOC**: 200-300

### Batch 3: SEARCH/FIND Tools
**Target**: After Batches 1-2  
**Scope**:
- [ ] Enhance grep with structured output
- [ ] Enhance find with metadata (file type, modified)
- [ ] Add filtering options

**Estimated LOC**: 400-500

---

## PR Plan

### PR 1: Read Selector Parser (Phase 1a) ✅
**Status**: Merged  
**Scope**: Selector parsing logic  
**Files**: `line_range_selector.rs` + lib.rs exports  
**Tests**: 13 unit tests

### PR 2: Read Selector Integration (Phase 1b-1c)
**Status**: Next  
**Scope**: ReadFileInput/Output update + integration  
**Files**: `tools/lib.rs`, `file_ops.rs`, tests  
**Size**: ~1000 LOC

### PR 3: EDIT Tool Enhancements (Batch 2)
**Status**: Planned  
**Scope**: Output format enhancements  
**Size**: ~200 LOC

### PR 4: SEARCH/FIND Tools (Batch 3)
**Status**: Planned  
**Scope**: Output format & filtering  
**Size**: ~400 LOC

---

## Testing Strategy

Each phase includes:
1. **Unit Tests**: Core functions (parsing, selection, formatting)
2. **Integration Tests**: End-to-end tool invocation
3. **Parity Tests**: Compare outputs against GJC reference (manual)

### Test Files
- `rust/crates/runtime/src/line_range_selector.rs:tests` ✅ (13 tests)
- `rust/crates/runtime/src/file_ops.rs:tests` (to be extended)
- `rust/crates/tools/src/lib.rs:tests` (to be extended)

---

## Risk Assessment

| Risk | Mitigation | Priority |
|------|-----------|----------|
| Large refactor breaks existing tests | Keep changes isolated, extend existing structs | High |
| GJC semantics unclear | Reference gajae-code/tools/read.ts directly | High |
| Performance regression | Benchmark file ops against baseline | Medium |
| Windows path handling | Use existing normalize_for_comparison logic | Medium |

---

## References

- **GJC Read Tool**: `/home/bellman/Workspace/gajae-code/packages/coding-agent/src/tools/read.ts`
- **Gap Matrix**: `docs/gjc-compat/gap-matrix.md`
- **Semantics Doc**: `docs/gjc-compat/tool-semantics-phase1.md`
- **Current Implementation**: `rust/crates/runtime/src/file_ops.rs:713`
- **Tool Input Schema**: `rust/crates/tools/src/lib.rs:2884`

---

## Next Steps

1. **Immediate** (this turn): Implement Phase 1b (ReadFileInput/Output integration)
2. **Short-term** (next commit): Add Phase 1c (truncation footers)
3. **Medium-term**: Batch 2 (EDIT tool) and Batch 3 (SEARCH/FIND)
4. **Verification**: Manual parity testing against GJC reference
5. **Merge**: Create focused PRs, merge when checks pass
