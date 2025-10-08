---
description: "Verify code according to best practices and industry standards"
---

Please review the current codebase according to:

1. **Best Practices**: Check for idiomatic Rust patterns, proper error handling, and code organization
2. **Performance**: Identify potential bottlenecks, unnecessary allocations, or inefficient algorithms
3. **Audio Quality**: Verify adherence to DSP best practices (especially for hot path optimization)
4. **Safety**: Look for potential panics, unwraps, or unsafe code that could be improved
5. **Modern Rust**: Ensure we're using current Rust idioms and features appropriately
6. **Project Architecture**: Verify compliance with the processing rate architecture, performance considerations, and design guidelines specified in CLAUDE.md

Focus on the most critical issues first. Provide specific file:line references for each finding.

$ARGUMENTS
