# MCP Server Integration Guide

> Build Model Context Protocol servers with prompt-compress integration

## Table of Contents

1. [What is MCP?](#what-is-mcp)
2. [Why MCP + prompt-compress?](#why-mcp--prompt-compress)
3. [Quick Start](#quick-start)
4. [MCP Server Implementation](#mcp-server-implementation)
5. [Tools](#tools)
6. [Resources](#resources)
7. [Prompts](#prompts)
8. [Advanced Features](#advanced-features)
9. [Deployment](#deployment)
10. [Examples](#examples)

---

## What is MCP?

[Model Context Protocol (MCP)](https://modelcontextprotocol.io/) is Anthropic's open standard for connecting AI assistants to external tools, data sources, and services. It enables:

- **Tools**: Function calls that Claude can invoke
- **Resources**: Files, databases, or APIs that Claude can access
- **Prompts**: Pre-built prompt templates
- **Sampling**: Let Claude control other LLM calls

### MCP Architecture

```
Claude Desktop/API
       ↓
MCP Protocol (stdio/HTTP)
       ↓
MCP Server (your code)
       ↓
External Services (APIs, DBs, etc.)
```

---

## Why MCP + prompt-compress?

Integrating `prompt-compress` as an MCP server provides:

1. **Token Savings in Real-Time**: Claude optimizes prompts before sending to LLMs
2. **Workflow Integration**: Automatically compress prompts in multi-agent systems
3. **Cost Optimization**: Reduce API costs by 15-40% on verbose prompts
4. **Quality Control**: Bayesian confidence ensures semantic preservation
5. **Multi-Language Support**: Automatically select optimal language forms

### Use Cases

- **Claude Desktop**: Optimize prompts during conversations
- **CI/CD**: Compress test prompts in automated pipelines
- **Content Management**: Optimize user-submitted prompts
- **Multi-Agent Systems**: Compress inter-agent messages
- **Analytics**: Track token savings across your organization

---

## Quick Start

### Prerequisites

```bash
# Install Node.js (v18+)
node --version

# Install MCP SDK
npm install @modelcontextprotocol/sdk

# Start prompt-compress API server
cargo run --bin prompt-compress-server
# API available at http://localhost:8080
```

### Minimal MCP Server

Create `mcp-server.js`:

```javascript
#!/usr/bin/env node
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { CallToolRequestSchema, ListToolsRequestSchema } from '@modelcontextprotocol/sdk/types.js';
import axios from 'axios';

const API_BASE = process.env.PROMPT_COMPRESS_API || 'http://localhost:8080/api/v1';

// Create MCP server
const server = new Server(
  {
    name: 'prompt-compress',
    version: '1.0.0',
  },
  {
    capabilities: {
      tools: {},
    },
  }
);

// List available tools
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      {
        name: 'optimize_prompt',
        description: 'Optimize a verbose prompt by removing boilerplate and compressing text',
        inputSchema: {
          type: 'object',
          properties: {
            prompt: {
              type: 'string',
              description: 'The prompt to optimize',
            },
            output_language: {
              type: 'string',
              description: 'Output language: english or mandarin',
              enum: ['english', 'mandarin'],
              default: 'english',
            },
            aggressive_mode: {
              type: 'boolean',
              description: 'Use aggressive compression (lower confidence threshold)',
              default: false,
            },
          },
          required: ['prompt'],
        },
      },
    ],
  };
});

// Handle tool calls
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  if (name === 'optimize_prompt') {
    const { prompt, output_language = 'english', aggressive_mode = false } = args;

    try {
      const response = await axios.post(`${API_BASE}/optimize`, {
        prompt,
        output_language,
        aggressive_mode,
        confidence_threshold: aggressive_mode ? 0.70 : 0.85,
      });

      const result = response.data.result;

      return {
        content: [
          {
            type: 'text',
            text: `Optimized Prompt:\n${result.optimized_prompt}\n\nOriginal: ${result.original_tokens} tokens\nOptimized: ${result.optimized_tokens} tokens\nSavings: ${result.token_savings} tokens (${result.savings_percentage.toFixed(1)}%)`,
          },
        ],
      };
    } catch (error) {
      return {
        content: [
          {
            type: 'text',
            text: `Error: ${error.message}`,
          },
        ],
        isError: true,
      };
    }
  }

  throw new Error(`Unknown tool: ${name}`);
});

// Start server
const transport = new StdioServerTransport();
await server.connect(transport);

console.error('Prompt-compress MCP server running on stdio');
```

### Make Executable

```bash
chmod +x mcp-server.js
```

### Configure Claude Desktop

Edit `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or `%APPDATA%\Claude\claude_desktop_config.json` (Windows):

```json
{
  "mcpServers": {
    "prompt-compress": {
      "command": "node",
      "args": ["/absolute/path/to/mcp-server.js"],
      "env": {
        "PROMPT_COMPRESS_API": "http://localhost:8080/api/v1"
      }
    }
  }
}
```

### Test in Claude Desktop

Restart Claude Desktop and try:

```
User: Optimize this prompt: "I would really appreciate if you could please take the time to analyze this code carefully and provide a very detailed explanation."

Claude: I'll use the optimize_prompt tool to compress this prompt.

[Uses optimize_prompt tool]

Optimized Prompt:
Analyze this code: detailed explanation.

Original: 24 tokens
Optimized: 6 tokens
Savings: 18 tokens (75.0%)
```

---

## MCP Server Implementation

### Full-Featured Server

`prompt-compress-mcp.js`:

```javascript
#!/usr/bin/env node
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
  ListResourcesRequestSchema,
  ReadResourceRequestSchema,
  ListPromptsRequestSchema,
  GetPromptRequestSchema,
} from '@modelcontextprotocol/sdk/types.js';
import axios from 'axios';

const API_BASE = process.env.PROMPT_COMPRESS_API || 'http://localhost:8080/api/v1';

const server = new Server(
  {
    name: 'prompt-compress',
    version: '1.0.0',
  },
  {
    capabilities: {
      tools: {},
      resources: {},
      prompts: {},
    },
  }
);

// ============================================================================
// TOOLS
// ============================================================================

server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      {
        name: 'optimize_prompt',
        description: 'Optimize a verbose prompt by removing boilerplate and compressing text',
        inputSchema: {
          type: 'object',
          properties: {
            prompt: { type: 'string', description: 'The prompt to optimize' },
            output_language: { type: 'string', enum: ['english', 'mandarin'], default: 'english' },
            aggressive_mode: { type: 'boolean', default: false },
          },
          required: ['prompt'],
        },
      },
      {
        name: 'analyze_prompt',
        description: 'Analyze optimization opportunities without applying them',
        inputSchema: {
          type: 'object',
          properties: {
            prompt: { type: 'string', description: 'The prompt to analyze' },
          },
          required: ['prompt'],
        },
      },
      {
        name: 'batch_optimize',
        description: 'Optimize multiple prompts at once',
        inputSchema: {
          type: 'object',
          properties: {
            prompts: { type: 'array', items: { type: 'string' }, description: 'Array of prompts' },
            output_language: { type: 'string', enum: ['english', 'mandarin'], default: 'english' },
          },
          required: ['prompts'],
        },
      },
    ],
  };
});

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  switch (name) {
    case 'optimize_prompt': {
      const { prompt, output_language = 'english', aggressive_mode = false } = args;
      const response = await axios.post(`${API_BASE}/optimize`, {
        prompt,
        output_language,
        aggressive_mode,
        confidence_threshold: aggressive_mode ? 0.70 : 0.85,
      });
      const result = response.data.result;
      return {
        content: [
          {
            type: 'text',
            text: `**Optimized Prompt:**\n\`\`\`\n${result.optimized_prompt}\n\`\`\`\n\n**Savings:** ${result.token_savings} tokens (${result.savings_percentage.toFixed(1)}%)\n\n**Details:**\n- Original: ${result.original_tokens} tokens\n- Optimized: ${result.optimized_tokens} tokens\n- Optimizations applied: ${result.optimizations.length}`,
          },
        ],
      };
    }

    case 'analyze_prompt': {
      const { prompt } = args;
      const response = await axios.post(`${API_BASE}/analyze`, {
        prompt,
        output_language: 'english',
      });
      const result = response.data.result;
      const optimizations = result.optimizations
        .map((opt) => `- **${opt.optimization_type}**: "${opt.original_text}" → "${opt.optimized_text}" (${opt.token_savings} tokens, confidence: ${(opt.confidence * 100).toFixed(0)}%)`)
        .join('\n');
      return {
        content: [
          {
            type: 'text',
            text: `**Optimization Opportunities:**\n\n${optimizations}\n\n**Total Potential Savings:** ${result.token_savings} tokens (${result.savings_percentage.toFixed(1)}%)`,
          },
        ],
      };
    }

    case 'batch_optimize': {
      const { prompts, output_language = 'english' } = args;
      const results = await Promise.all(
        prompts.map(async (prompt) => {
          const response = await axios.post(`${API_BASE}/optimize`, {
            prompt,
            output_language,
            confidence_threshold: 0.85,
          });
          return response.data.result;
        })
      );
      const totalSavings = results.reduce((sum, r) => sum + r.token_savings, 0);
      const avgSavingsPercent = results.reduce((sum, r) => sum + r.savings_percentage, 0) / results.length;
      const optimized = results.map((r, i) => `${i + 1}. ${r.optimized_prompt}`).join('\n');
      return {
        content: [
          {
            type: 'text',
            text: `**Batch Optimization Complete**\n\n${optimized}\n\n**Total Savings:** ${totalSavings} tokens (avg ${avgSavingsPercent.toFixed(1)}% per prompt)`,
          },
        ],
      };
    }

    default:
      throw new Error(`Unknown tool: ${name}`);
  }
});

// ============================================================================
// RESOURCES
// ============================================================================

server.setRequestHandler(ListResourcesRequestSchema, async () => {
  return {
    resources: [
      {
        uri: 'pattern://boilerplate',
        name: 'Boilerplate Patterns',
        description: 'List of boilerplate removal patterns',
        mimeType: 'text/plain',
      },
      {
        uri: 'pattern://filler',
        name: 'Filler Word Patterns',
        description: 'List of filler word removal patterns',
        mimeType: 'text/plain',
      },
      {
        uri: 'stats://optimizer',
        name: 'Optimizer Statistics',
        description: 'Usage statistics and performance metrics',
        mimeType: 'application/json',
      },
    ],
  };
});

server.setRequestHandler(ReadResourceRequestSchema, async (request) => {
  const { uri } = request.params;

  if (uri === 'pattern://boilerplate') {
    // Fetch boilerplate patterns from API or database
    const patterns = [
      'I would (really )?appreciate (it )?if you could',
      'Please make sure to',
      'Thank you (so much )?in advance',
      'If you don\\'t mind,?',
      'Could you please',
    ];
    return {
      contents: [
        {
          uri,
          mimeType: 'text/plain',
          text: patterns.join('\n'),
        },
      ],
    };
  }

  if (uri === 'pattern://filler') {
    const fillers = ['really', 'very', 'quite', 'just', 'actually', 'basically', 'essentially'];
    return {
      contents: [
        {
          uri,
          mimeType: 'text/plain',
          text: fillers.join('\n'),
        },
      ],
    };
  }

  if (uri === 'stats://optimizer') {
    // In production, fetch from database
    const stats = {
      total_optimizations: 1234,
      total_tokens_saved: 45678,
      average_savings_percent: 32.5,
      most_common_pattern: 'boilerplate',
    };
    return {
      contents: [
        {
          uri,
          mimeType: 'application/json',
          text: JSON.stringify(stats, null, 2),
        },
      ],
    };
  }

  throw new Error(`Unknown resource: ${uri}`);
});

// ============================================================================
// PROMPTS
// ============================================================================

server.setRequestHandler(ListPromptsRequestSchema, async () => {
  return {
    prompts: [
      {
        name: 'code_review',
        description: 'Optimized code review prompt template',
      },
      {
        name: 'bug_analysis',
        description: 'Optimized bug analysis prompt template',
      },
    ],
  };
});

server.setRequestHandler(GetPromptRequestSchema, async (request) => {
  const { name } = request.params;

  if (name === 'code_review') {
    return {
      messages: [
        {
          role: 'user',
          content: {
            type: 'text',
            text: 'Analyze code: functionality, edge cases, performance. Identify bugs. Suggest improvements.\n\n[output_language: english]',
          },
        },
      ],
    };
  }

  if (name === 'bug_analysis') {
    return {
      messages: [
        {
          role: 'user',
          content: {
            type: 'text',
            text: 'Debug issue: reproduce, root cause, fix. Test edge cases.\n\n[output_language: english]',
          },
        },
      ],
    };
  }

  throw new Error(`Unknown prompt: ${name}`);
});

// Start server
const transport = new StdioServerTransport();
await server.connect(transport);
console.error('Prompt-compress MCP server running');
```

---

## Tools

### optimize_prompt

**Purpose**: Optimize a verbose prompt

**Input:**
```json
{
  "prompt": "I would really appreciate if you could help me with this task.",
  "output_language": "english",
  "aggressive_mode": false
}
```

**Output:**
```
Optimized Prompt:
Help me with this task.

Savings: 8 tokens (44.4%)
```

### analyze_prompt

**Purpose**: Analyze optimization opportunities without applying them

**Input:**
```json
{
  "prompt": "I would really appreciate if you could please analyze this code carefully."
}
```

**Output:**
```
Optimization Opportunities:

- BoilerplateRemoval: "I would really appreciate if you could please" → "" (9 tokens, confidence: 97%)
- FillerRemoval: "carefully" → "" (1 token, confidence: 85%)

Total Potential Savings: 10 tokens (50.0%)
```

### batch_optimize

**Purpose**: Optimize multiple prompts at once

**Input:**
```json
{
  "prompts": [
    "Please analyze this code.",
    "I would appreciate your help with this bug.",
    "Could you please review this PR?"
  ],
  "output_language": "english"
}
```

**Output:**
```
Batch Optimization Complete

1. Analyze this code.
2. Help with this bug.
3. Review this PR.

Total Savings: 12 tokens (avg 33.3% per prompt)
```

---

## Resources

Resources are read-only data that Claude can access.

### pattern://boilerplate

**Purpose**: List boilerplate patterns

**Usage in Claude:**
```
User: Show me the boilerplate patterns

Claude: [Reads pattern://boilerplate resource]
Here are the boilerplate removal patterns:
- I would (really)? appreciate (it)? if you could
- Please make sure to
- Thank you (so much)? in advance
...
```

### stats://optimizer

**Purpose**: Usage statistics

**Usage in Claude:**
```
User: Show optimizer stats

Claude: [Reads stats://optimizer resource]
Optimizer Statistics:
- Total optimizations: 1,234
- Total tokens saved: 45,678
- Average savings: 32.5%
- Most common pattern: boilerplate
```

---

## Prompts

Pre-built prompt templates that Claude can use.

### code_review

**Template:**
```
Analyze code: functionality, edge cases, performance. Identify bugs. Suggest improvements.

[output_language: english]
```

**Usage in Claude:**
```
User: Review this code using the code_review prompt

Claude: [Uses code_review prompt template]
I'll analyze the code for functionality, edge cases, performance, bugs, and suggest improvements.
```

---

## Advanced Features

### Streaming Responses

For long prompts, stream the optimization progress:

```javascript
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  if (request.params.name === 'optimize_prompt_stream') {
    const { prompt } = request.params.arguments;

    // Send progress updates
    await server.sendNotification({
      method: 'notifications/progress',
      params: {
        progressToken: 'opt-123',
        progress: 0,
        total: 100,
      },
    });

    // Optimize
    const response = await axios.post(`${API_BASE}/optimize`, { prompt });

    await server.sendNotification({
      method: 'notifications/progress',
      params: {
        progressToken: 'opt-123',
        progress: 100,
        total: 100,
      },
    });

    return { content: [{ type: 'text', text: response.data.result.optimized_prompt }] };
  }
});
```

### Error Handling

```javascript
try {
  const response = await axios.post(`${API_BASE}/optimize`, { prompt });
  return { content: [{ type: 'text', text: response.data.result.optimized_prompt }] };
} catch (error) {
  if (error.response?.status === 503) {
    return {
      content: [{ type: 'text', text: 'Service temporarily unavailable. Please try again.' }],
      isError: true,
    };
  }
  throw error;
}
```

### Caching

Cache optimizations to reduce API calls:

```javascript
const cache = new Map();

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { prompt } = request.params.arguments;

  // Check cache
  if (cache.has(prompt)) {
    return { content: [{ type: 'text', text: cache.get(prompt) }] };
  }

  // Optimize
  const response = await axios.post(`${API_BASE}/optimize`, { prompt });
  const optimized = response.data.result.optimized_prompt;

  // Cache result
  cache.set(prompt, optimized);

  return { content: [{ type: 'text', text: optimized }] };
});
```

---

## Deployment

### Docker

`Dockerfile`:

```dockerfile
FROM node:18-alpine

WORKDIR /app

COPY package*.json ./
RUN npm install

COPY mcp-server.js ./

CMD ["node", "mcp-server.js"]
```

`docker-compose.yml`:

```yaml
version: '3.8'

services:
  prompt-compress-api:
    build: ../  # Rust API
    ports:
      - "8080:8080"

  mcp-server:
    build: .
    environment:
      PROMPT_COMPRESS_API: http://prompt-compress-api:8080/api/v1
    stdin_open: true
    tty: true
```

### Remote MCP Server (HTTP)

MCP supports HTTP transport for remote servers:

```javascript
import { SSEServerTransport } from '@modelcontextprotocol/sdk/server/sse.js';
import express from 'express';

const app = express();
const server = new Server({ name: 'prompt-compress', version: '1.0.0' });

// ... (register tools, resources, prompts)

app.get('/mcp', async (req, res) => {
  const transport = new SSEServerTransport('/mcp/messages', res);
  await server.connect(transport);
});

app.listen(3000, () => {
  console.log('MCP server running on http://localhost:3000/mcp');
});
```

**Claude Desktop Config:**

```json
{
  "mcpServers": {
    "prompt-compress": {
      "url": "http://localhost:3000/mcp"
    }
  }
}
```

---

## Examples

### Example 1: Code Review Assistant

```javascript
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  if (request.params.name === 'review_code') {
    const { code, language } = request.params.arguments;

    // Create verbose review prompt
    const verbosePrompt = `I would really appreciate if you could please carefully analyze this ${language} code and provide a very thorough and detailed review. Please make sure to check for bugs, performance issues, and best practices. Thank you in advance!

Code:
\`\`\`${language}
${code}
\`\`\``;

    // Optimize
    const response = await axios.post(`${API_BASE}/optimize`, {
      prompt: verbosePrompt,
      output_language: 'english',
      aggressive_mode: true,
    });

    return {
      content: [
        {
          type: 'text',
          text: `Optimized review prompt:\n${response.data.result.optimized_prompt}\n\nSavings: ${response.data.result.token_savings} tokens`,
        },
      ],
    };
  }
});
```

### Example 2: Multi-Agent Communication

```javascript
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  if (request.params.name === 'agent_message') {
    const { from_agent, to_agent, message } = request.params.arguments;

    // Optimize inter-agent message
    const response = await axios.post(`${API_BASE}/optimize`, {
      prompt: `From ${from_agent} to ${to_agent}: ${message}`,
      output_language: 'english',
      aggressive_mode: true,
    });

    // Send optimized message
    const optimized = response.data.result.optimized_prompt;
    // ... (send to agent)

    return {
      content: [
        {
          type: 'text',
          text: `Message sent: ${optimized}\nSavings: ${response.data.result.token_savings} tokens`,
        },
      ],
    };
  }
});
```

### Example 3: Cost Tracking

```javascript
let totalSavings = 0;
let totalRequests = 0;

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  if (request.params.name === 'optimize_prompt') {
    const response = await axios.post(`${API_BASE}/optimize`, request.params.arguments);

    totalSavings += response.data.result.token_savings;
    totalRequests += 1;

    return {
      content: [
        {
          type: 'text',
          text: `Optimized!\n\nThis request: ${response.data.result.token_savings} tokens saved\nTotal session: ${totalSavings} tokens saved across ${totalRequests} requests`,
        },
      ],
    };
  }

  if (request.params.name === 'get_savings_report') {
    const avgSavings = totalSavings / totalRequests;
    const estimatedCost = (totalSavings * 0.00001).toFixed(4); // $0.01 per 1K tokens
    return {
      content: [
        {
          type: 'text',
          text: `Session Report:\n- Total requests: ${totalRequests}\n- Total tokens saved: ${totalSavings}\n- Average savings: ${avgSavings.toFixed(1)} tokens/request\n- Estimated cost savings: $${estimatedCost}`,
        },
      ],
    };
  }
});
```

---

## Troubleshooting

### Server Not Showing Up in Claude

1. Check config file location:
   - macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
   - Windows: `%APPDATA%\Claude\claude_desktop_config.json`

2. Validate JSON syntax:
   ```bash
   jq . ~/Library/Application\ Support/Claude/claude_desktop_config.json
   ```

3. Check server logs:
   - Look for stderr output in Claude Desktop console
   - Add `console.error('Server started')` to verify execution

4. Test manually:
   ```bash
   echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | node mcp-server.js
   ```

### API Connection Errors

1. Verify API is running:
   ```bash
   curl http://localhost:8080/api/v1/health
   ```

2. Check environment variable:
   ```json
   "env": {
     "PROMPT_COMPRESS_API": "http://localhost:8080/api/v1"
   }
   ```

3. Test with absolute URL:
   ```javascript
   const API_BASE = 'http://127.0.0.1:8080/api/v1';
   ```

---

**Next Steps:**
- [Claude Skills Guide](./CLAUDE-SKILLS-GUIDE.md)
- [API Documentation](./CONSOLIDATED-ARCHITECTURE.md)
- [Contributors Guide](./CONTRIBUTORS.md)
