---
description: "Targeted optimization analysis for specific files or functions"
---

$ARGUMENTS

**Task**: Apply the comprehensive 7-point optimization analysis from `.claude/commands/optimize.md` to the target specified above.

**Scope Interpretation**:
- If only a file is specified (e.g., `@src/engine.rs`): Analyze the entire file for optimization opportunities
- If a file + identifier is specified (e.g., `@src/engine.rs render_sample`): Focus specifically on optimizing that function/method/struct/variable

**Analysis Framework** (from optimize.md):
1. Algorithm Research - state-of-the-art techniques
2. Hot Path Analysis - verify no expensive ops at 44.1kHz
3. Caching Opportunities - reciprocals, coefficients, lookup tables
4. Inlining Strategy - #[inline] on hot path
5. Math Library Usage - micromath vs libm
6. Division Elimination - use reciprocal multiplication
7. Memory & Allocation - no heap allocations in audio callbacks

Provide specific, actionable optimizations with before/after code examples and estimated performance impact for the ESP32-S3 XTensa LX7 architecture.
