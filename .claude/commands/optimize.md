---
description: "Deep optimization pass for embedded audio performance"
argument-hint: [optional: @file or @file identifier]
---

Please perform a comprehensive optimization review$ARGUMENTS of the code, with special focus on embedded constraints (ESP32-S3 dual-core XTensa LX7):

**Scope**:
- If no target specified: Analyze the entire codebase for optimization opportunities
- If only a file specified (e.g., `@src/engine.rs`): Focus optimization analysis on that file
- If file + identifier specified (e.g., `@src/engine.rs render_sample`): Focus specifically on that function/method/struct

**Analysis Framework**:

1. **Algorithm Research**: Search for state-of-the-art algorithms and optimization techniques related to the current implementation. Are there more efficient approaches used in professional audio DSP?

2. **Hot Path Analysis**: Verify that expensive calculations (transcendental functions, divisions, allocations) are NOT in the audio-rate path (44.1kHz). Move them to buffer-rate (~172Hz) or control-rate (50Hz) if possible.

3. **Caching Opportunities**: Identify values that can be pre-computed at buffer-rate or control-rate rather than recalculated every sample. Look for reciprocals, coefficients, and lookup tables.

4. **Inlining Strategy**: Ensure `#[inline]` or `#[inline(always)]` is applied to small, frequently-called functions in the audio hot path. Verify larger functions are NOT over-inlined.

5. **Math Library Usage**: Check if `micromath` crate can replace `libm` for transcendental functions where appropriate (faster approximations for embedded).

6. **Division Elimination**: Replace division operations with multiplication by reciprocals wherever possible (division is 10-40 cycles vs 1-2 for multiplication on XTensa).

7. **Memory & Allocation**: Verify no heap allocations occur in audio callbacks. Check for unnecessary copies or large stack frames.

Provide specific optimizations with before/after code examples and estimated performance impact.
