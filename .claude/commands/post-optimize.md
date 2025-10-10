---
description: "Thoroughly verify optimization findings by tracing code"
---

For each optimization finding from the previous analysis, perform a rigorous verification:

**Verification Process:**

1. **List All Findings**: Enumerate each optimization issue/opportunity that was identified

2. **Deep Trace for Each Finding**:
   - Read all relevant source files completely
   - Trace through every code reference and call chain
   - Verify the actual behavior matches the claimed issue
   - Check if mitigating factors exist that weren't considered

3. **Evidence Collection**:
   - Quote specific line numbers and code snippets
   - Show the actual call path that demonstrates the issue
   - Identify any assumptions that might be incorrect

4. **Classification**:
   - **CONFIRMED**: Issue verified with evidence
   - **REFUTED**: Analysis was incorrect, explain why
   - **UNCERTAIN**: Need more information or context

5. **Priority Adjustment**: Based on verification, re-rank findings by actual impact

**Critical Questions for Each Finding:**
- Does this code path actually execute at audio-rate/buffer-rate/control-rate?
- Are there caching mechanisms I missed?
- Is the compiler already optimizing this?
- What's the actual call frequency?

Be thorough and skeptical. It's better to refute a false finding than to implement unnecessary changes.

$ARGUMENTS
