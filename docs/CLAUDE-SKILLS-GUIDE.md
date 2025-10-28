# Claude Skills Integration Guide

> Create reusable Claude skills powered by prompt-compress

## Table of Contents

1. [What are Claude Skills?](#what-are-claude-skills)
2. [Skill Architecture](#skill-architecture)
3. [Creating Your First Skill](#creating-your-first-skill)
4. [Advanced Skills](#advanced-skills)
5. [Skill Composition](#skill-composition)
6. [Testing Skills](#testing-skills)
7. [Deployment](#deployment)
8. [Examples](#examples)

---

## What are Claude Skills?

Claude Skills are reusable capabilities that can be invoked during conversations. They combine:
- **Trigger Patterns**: When to activate
- **Instructions**: What to do
- **Tools**: External tools to use (via MCP)
- **Context**: Knowledge and data to reference

### Why Create Skills?

1. **Consistency**: Standardize common workflows
2. **Efficiency**: Reduce token usage with optimized prompts
3. **Reusability**: Share skills across projects
4. **Integration**: Connect to external tools and APIs

### prompt-compress Skills

By integrating prompt-compress, skills can:
- Automatically optimize verbose user inputs
- Reduce token costs by 15-40%
- Maintain semantic meaning with Bayesian confidence
- Support multilingual optimization

---

## Skill Architecture

```
User Message
     ↓
[Trigger Detection] → Match pattern
     ↓
[Skill Activation] → Load instructions + tools
     ↓
[prompt-compress] → Optimize user input (optional)
     ↓
[Execute Tools] → Call MCP tools
     ↓
[Generate Response] → Use optimized context
     ↓
Claude Response
```

---

## Creating Your First Skill

### Step 1: Basic Skill Definition

Create `.claude/skills/prompt-optimizer.yaml`:

```yaml
name: prompt_optimizer
version: 1.0.0
description: Optimize verbose prompts by removing boilerplate and compressing text

# When to activate this skill
triggers:
  - pattern: "optimize (this|my|the) prompt"
  - pattern: "compress (this|my|the) prompt"
  - pattern: "remove boilerplate from"
  - pattern: "make this prompt more concise"

# What Claude should do
instructions: |
  When the user asks you to optimize a prompt:

  1. Extract the prompt to optimize (usually in quotes, code blocks, or immediately after the request)
  2. Use the `optimize_prompt` tool from the prompt-compress MCP server
  3. Show both the original and optimized versions
  4. Display token savings and optimization details
  5. Ask if the user wants to make any adjustments

  Be concise and focus on the optimization results.

# External tools this skill can use
tools:
  - name: optimize_prompt
    server: prompt-compress
    description: Optimize a verbose prompt by removing boilerplate

# Examples for testing
examples:
  - input: |
      Optimize this prompt: "I would really appreciate if you could please help me with this task."
    output: |
      Original: "I would really appreciate if you could please help me with this task."
      Optimized: "Help me with this task."

      Savings: 8 tokens (44.4%)

      Would you like to use this optimized version?

  - input: |
      Make this more concise: "I need you to carefully analyze this code and provide a very detailed explanation."
    output: |
      Original: "I need you to carefully analyze this code and provide a very detailed explanation."
      Optimized: "Analyze this code: detailed explanation."

      Savings: 10 tokens (55.6%)
```

### Step 2: Configure MCP Server

Ensure `prompt-compress` MCP server is configured in `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "prompt-compress": {
      "command": "node",
      "args": ["/path/to/mcp-server.js"],
      "env": {
        "PROMPT_COMPRESS_API": "http://localhost:8080/api/v1"
      }
    }
  }
}
```

### Step 3: Test the Skill

In Claude Desktop:

```
User: Optimize this prompt: "I would really appreciate if you could please take the time to analyze this code carefully."

Claude: [Activates prompt_optimizer skill]

I'll optimize that prompt for you.

[Uses optimize_prompt tool]

Original: "I would really appreciate if you could please take the time to analyze this code carefully."
Optimized: "Analyze this code."

Savings: 14 tokens (70.0%)

The optimized version is much more concise while preserving the core instruction. Would you like to use this version or make any adjustments?
```

---

## Advanced Skills

### Context-Aware Optimization

`.claude/skills/smart-optimizer.yaml`:

```yaml
name: smart_optimizer
version: 1.0.0
description: Context-aware prompt optimization with language detection

triggers:
  - pattern: "optimize|compress|simplify"

instructions: |
  Analyze the conversation context to determine:

  1. **Output Language**:
     - If the user has been writing in Mandarin, use `output_language: "mandarin"`
     - Otherwise, use `output_language: "english"`

  2. **Aggression Level**:
     - For casual conversations: `aggressive_mode: true` (70% confidence threshold)
     - For formal/technical: `aggressive_mode: false` (85% confidence threshold)

  3. **Protected Regions**:
     - If code blocks, URLs, or technical terms are present, they'll be automatically preserved

  Then optimize using these context-aware settings.

tools:
  - name: optimize_prompt
    server: prompt-compress

  - name: analyze_prompt
    server: prompt-compress
    description: Analyze without applying (for review)

examples:
  - input: |
      User context: Technical discussion about Python async/await
      Request: "Optimize: I would really appreciate if you could explain async programming."
    output: |
      Detected context: Technical, formal
      Settings: aggressive_mode=false (85% threshold)

      Optimized: "Explain async programming."
      Savings: 6 tokens (46.2%)

  - input: |
      User context: Casual conversation in Mandarin
      Request: "优化这个提示：请你帮我分析这段代码"
    output: |
      检测上下文：中文对话
      设置：output_language="mandarin", aggressive_mode=true

      优化后："分析代码"
      节省：4 tokens (57.1%)
```

### Batch Optimization Skill

`.claude/skills/batch-optimizer.yaml`:

```yaml
name: batch_optimizer
version: 1.0.0
description: Optimize multiple prompts at once

triggers:
  - pattern: "optimize (all|these|multiple) prompts?"
  - pattern: "batch optimize"

instructions: |
  When the user provides multiple prompts to optimize:

  1. Extract all prompts (numbered lists, bullet points, or separate quotes)
  2. Use the `batch_optimize` tool to optimize all at once
  3. Display a comparison table:
     - Original prompt
     - Optimized prompt
     - Token savings
     - Savings percentage
  4. Show total savings across all prompts

tools:
  - name: batch_optimize
    server: prompt-compress

examples:
  - input: |
      Optimize these prompts:
      1. "I would appreciate if you could help me debug this code."
      2. "Please make sure to check for performance issues."
      3. "Thank you in advance for your detailed analysis."
    output: |
      Batch Optimization Results:

      | # | Original | Optimized | Savings |
      |---|----------|-----------|---------|
      | 1 | I would appreciate... (11 tokens) | Debug this code. (4 tokens) | 7 (63.6%) |
      | 2 | Please make sure... (9 tokens) | Check performance. (3 tokens) | 6 (66.7%) |
      | 3 | Thank you in advance... (9 tokens) | (removed) | 9 (100%) |

      Total Savings: 22 tokens (73.3% average)
```

### Code Review Optimizer

`.claude/skills/code-review-optimizer.yaml`:

```yaml
name: code_review_optimizer
version: 1.0.0
description: Optimize code review prompts with technical context preservation

triggers:
  - pattern: "review (this|my) (code|pr|pull request)"
  - pattern: "code review"

instructions: |
  When the user requests a code review:

  1. Extract the code and review instructions
  2. Optimize the review prompt using `optimize_prompt` with:
     - `output_language: "english"`
     - `aggressive_mode: false` (preserve technical details)
  3. Protected regions ensure code, function names, and URLs are preserved
  4. Show the optimized review prompt
  5. Proceed with the code review using the optimized instructions

tools:
  - name: optimize_prompt
    server: prompt-compress

context:
  - type: resource
    uri: pattern://boilerplate
    description: Reference boilerplate patterns to remove

examples:
  - input: |
      Review this code:
      ```python
      def calculate_sum(a, b):
          return a + b
      ```
      I would really appreciate if you could please carefully analyze this function and check for any potential bugs or performance issues. Please make sure to be thorough and detailed in your review. Thank you!
    output: |
      Optimized Review Prompt:
      "Analyze function: bugs, performance issues.

      ```python
      def calculate_sum(a, b):
          return a + b
      ```

      [output_language: english]"

      Savings: 24 tokens (68.6%)
      Code block preserved: ✓

      Now proceeding with review...
```

---

## Skill Composition

Combine multiple skills for complex workflows.

### Master Optimization Skill

`.claude/skills/master-optimizer.yaml`:

```yaml
name: master_optimizer
version: 1.0.0
description: Comprehensive optimization with analysis and reporting

triggers:
  - pattern: "full optimization"
  - pattern: "comprehensive optimize"

instructions: |
  Perform comprehensive optimization:

  1. **Analyze**: Use `analyze_prompt` to identify optimization opportunities
  2. **Review**: Show user the potential optimizations and ask for confirmation
  3. **Optimize**: Apply optimizations using `optimize_prompt`
  4. **Report**: Generate detailed report with:
     - Token savings breakdown by optimization type
     - Confidence scores
     - Suggestions for further improvement
  5. **Export**: Offer to save the optimized prompt to a file

tools:
  - name: analyze_prompt
    server: prompt-compress

  - name: optimize_prompt
    server: prompt-compress

sub_skills:
  - prompt_optimizer
  - batch_optimizer

examples:
  - input: |
      Full optimization: "I would really appreciate if you could please take the time to carefully review and analyze this code snippet. I want you to provide a very thorough and detailed explanation of what it does, how it works, and why it was implemented this way."
    output: |
      **Phase 1: Analysis**

      Optimization Opportunities Found:
      - Boilerplate: "I would really appreciate if you could please" (9 tokens, 97% confidence)
      - Filler: "really", "very" (2 tokens, 89% confidence)
      - Redundancy: "review and analyze" → "analyze" (2 tokens, 92% confidence)
      - Verbose: "provide a very thorough and detailed explanation" → "explain" (7 tokens, 88% confidence)

      Total Potential Savings: 20 tokens (62.5%)

      Proceed with optimization? [Y/n]

      **Phase 2: Optimization**

      Original (32 tokens):
      "I would really appreciate if you could please take the time to carefully review and analyze this code snippet. I want you to provide a very thorough and detailed explanation of what it does, how it works, and why it was implemented this way."

      Optimized (12 tokens):
      "Analyze code snippet: functionality, implementation, rationale. Explain details."

      **Phase 3: Report**

      Optimization Summary:
      - Boilerplate removed: 9 tokens
      - Fillers removed: 2 tokens
      - Redundancy eliminated: 2 tokens
      - Verbose instructions compressed: 7 tokens

      Total Savings: 20 tokens (62.5%)
      Average Confidence: 91.5%

      Recommendations:
      - Consider using Mandarin for "explain details" → "详细说明" (saves 1 more token)

      Would you like to export this optimized prompt?
```

---

## Testing Skills

### Unit Testing

Create `tests/skills/test_prompt_optimizer.yaml`:

```yaml
test_suite: prompt_optimizer_tests
skill: prompt_optimizer

tests:
  - name: simple_boilerplate_removal
    input: 'Optimize: "I would appreciate if you could help me."'
    expected_savings_min: 5
    expected_savings_percent_min: 30.0

  - name: aggressive_compression
    input: 'Compress: "I would really appreciate if you could please take the time to carefully analyze this code and provide a very detailed explanation."'
    expected_savings_min: 15
    expected_savings_percent_min: 50.0

  - name: code_preservation
    input: |
      Optimize: "Please analyze this function:
      ```python
      def foo(): pass
      ```"
    assertions:
      - contains: "```python"
      - contains: "def foo(): pass"

  - name: multilingual
    input: 'Optimize with Mandarin: "Please provide a detailed explanation."'
    assertions:
      - contains: "详细"
      - savings_min: 2
```

### Integration Testing

```bash
# Test skill activation
claude-cli test-skill prompt_optimizer \
  --input "Optimize: I would appreciate if you could help me." \
  --expect-tool optimize_prompt

# Test batch
claude-cli test-skill batch_optimizer \
  --input "Optimize these: 1. Prompt A 2. Prompt B" \
  --expect-tool batch_optimize
```

### Manual Testing

In Claude Desktop:

```
User: Test the prompt_optimizer skill with: "I would really appreciate if you could please help me with this task."

Claude: [Activates prompt_optimizer skill]
[Tests optimization]
[Shows results]

Result: ✓ Skill working correctly
- Original: 15 tokens
- Optimized: 7 tokens
- Savings: 8 tokens (53.3%)
```

---

## Deployment

### Local Development

```bash
# Skill directory structure
.claude/
└── skills/
    ├── prompt-optimizer.yaml
    ├── batch-optimizer.yaml
    ├── code-review-optimizer.yaml
    └── master-optimizer.yaml
```

### Team Deployment

Share skills via Git repository:

```bash
# Clone team skills repo
git clone https://github.com/your-org/claude-skills.git ~/.claude/skills

# Or use symlinks
ln -s ~/projects/claude-skills ~/.claude/skills
```

### Enterprise Deployment

Host skills on internal server:

```yaml
# .claude/config.yaml
skill_repositories:
  - url: https://skills.company.com/api/skills
    auth: ${COMPANY_SKILLS_TOKEN}
    auto_update: true
```

---

## Examples

### Example 1: API Documentation Generator

`.claude/skills/api-doc-optimizer.yaml`:

```yaml
name: api_doc_optimizer
version: 1.0.0
description: Optimize API documentation prompts

triggers:
  - pattern: "document (this|the) api"
  - pattern: "generate api docs"

instructions: |
  When documenting an API:

  1. Extract API endpoint, method, parameters
  2. Create verbose documentation prompt
  3. Optimize using `optimize_prompt` with `aggressive_mode: false`
  4. Generate documentation using optimized prompt
  5. Show token savings

tools:
  - name: optimize_prompt
    server: prompt-compress

examples:
  - input: |
      Document this API:
      POST /api/v1/users
      Creates a new user
    output: |
      Original prompt (generated):
      "I would like you to please generate comprehensive API documentation for this endpoint. Please make sure to include all details about the request and response formats, parameters, error codes, and provide clear examples."

      Optimized (12 tokens saved):
      "Document API endpoint: request/response formats, parameters, error codes, examples."

      Generating documentation...
```

### Example 2: SQL Query Optimizer

`.claude/skills/sql-optimizer.yaml`:

```yaml
name: sql_optimizer
version: 1.0.0
description: Optimize SQL query review prompts

triggers:
  - pattern: "review (this|my) sql"
  - pattern: "optimize (this|my) query"

instructions: |
  For SQL query reviews:

  1. Preserve SQL query (protected region)
  2. Optimize review instructions using `optimize_prompt`
  3. Protected regions ensure SQL syntax is never modified
  4. Review query using optimized instructions

tools:
  - name: optimize_prompt
    server: prompt-compress

examples:
  - input: |
      Review this SQL:
      SELECT * FROM users WHERE age > 18;
      I would appreciate if you could check for performance issues and suggest improvements.
    output: |
      Optimized Review Prompt:
      "Review SQL: performance issues, improvements.
      ```sql
      SELECT * FROM users WHERE age > 18;
      ```"

      SQL preserved: ✓
      Savings: 8 tokens (47.1%)
```

### Example 3: Meeting Notes Summarizer

`.claude/skills/meeting-summarizer.yaml`:

```yaml
name: meeting_summarizer
version: 1.0.0
description: Optimize meeting notes summarization prompts

triggers:
  - pattern: "summarize (this|the) meeting"
  - pattern: "meeting notes summary"

instructions: |
  For meeting summaries:

  1. Extract meeting notes/transcript
  2. Create summarization prompt
  3. Optimize with `aggressive_mode: true` (casual context)
  4. Summarize using optimized prompt
  5. Format as bullet points

tools:
  - name: optimize_prompt
    server: prompt-compress

examples:
  - input: |
      Summarize this meeting:
      [Long meeting transcript...]
      Please provide a comprehensive summary with action items.
    output: |
      Optimized Prompt:
      "Summarize: key points, action items."

      Savings: 5 tokens

      Meeting Summary:
      - Key Decision: Launch new feature next quarter
      - Action Items:
        - @John: Prepare design mockups
        - @Sarah: Review technical requirements
```

---

## Best Practices

### 1. Skill Naming

- Use descriptive names: `code_review_optimizer`, not `optimizer1`
- Follow convention: `noun_verb` format
- Version your skills: `v1.0.0`, `v2.0.0`

### 2. Trigger Patterns

- Be specific: Avoid overly broad patterns
- Use regex for flexibility: `(this|my|the)`
- Test with real user inputs

### 3. Instructions

- Be explicit: Tell Claude exactly what to do
- Use numbered steps: Easier to follow
- Include edge cases: Handle errors gracefully

### 4. Tool Usage

- Minimize tool calls: Use batch operations
- Cache results: Avoid redundant API calls
- Handle errors: Provide fallback behavior

### 5. Testing

- Unit test each skill: Verify expected behavior
- Integration test with real data: Catch edge cases
- User test: Get feedback from actual users

---

## Troubleshooting

### Skill Not Activating

1. Check trigger patterns: Test with regex101.com
2. Verify skill is in `.claude/skills/` directory
3. Restart Claude Desktop
4. Check logs for errors

### Tool Not Found

1. Verify MCP server is running:
   ```bash
   curl http://localhost:8080/api/v1/health
   ```
2. Check `claude_desktop_config.json` has correct server config
3. Test MCP server manually:
   ```bash
   echo '{"method":"tools/list"}' | node mcp-server.js
   ```

### Poor Optimization Results

1. Adjust confidence threshold:
   ```yaml
   tools:
     - name: optimize_prompt
       arguments:
         confidence_threshold: 0.90  # Higher = more conservative
   ```
2. Use aggressive mode for casual prompts:
   ```yaml
   tools:
     - name: optimize_prompt
       arguments:
         aggressive_mode: true
   ```
3. Check protected regions are working:
   ```yaml
   instructions: |
     Verify code blocks, URLs, and technical terms are preserved.
   ```

---

## Resources

- [MCP Server Guide](./MCP-SERVER-GUIDE.md)
- [Contributors Guide](./CONTRIBUTORS.md)
- [API Documentation](./CONSOLIDATED-ARCHITECTURE.md)
- [Claude Code Documentation](https://docs.claude.com/)

---

**Happy skill building!**
