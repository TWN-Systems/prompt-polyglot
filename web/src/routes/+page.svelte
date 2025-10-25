<script lang="ts">
  import type { ActionData } from './$types';

  export let form: ActionData;

  const tokenizerOptions = [
    { label: 'GPT-4o', value: 'gpt-4o' },
    { label: 'GPT-3.5 Turbo', value: 'gpt-3.5-turbo' },
    { label: 'Claude 3.5 Sonnet', value: 'claude-3.5-sonnet' }
  ];

  const basePayload = {
    text: '',
    tokenizer_id: '',
    rules: {
      units: true,
      numbers: true,
      whitespace: true
    },
    policy: {
      protect_code: true,
      protect_json_keys: true,
      min_saving_pct: 10
    }
  };

  let payload = basePayload;

  $: payload = {
    ...basePayload,
    ...(form?.payload ?? {}),
    rules: {
      ...basePayload.rules,
      ...(form?.payload?.rules ?? {})
    },
    policy: {
      ...basePayload.policy,
      ...(form?.payload?.policy ?? {})
    }
  };
</script>

<svelte:head>
  <title>Prompt Polyglot Compression</title>
</svelte:head>

<main class="container">
  <h1>Prompt Compression Playground</h1>
  <p class="description">
    Experiment with compression settings and send requests to the Prompt Polyglot API.
  </p>

  {#if form?.error}
    <div class="alert error">{form.error}</div>
  {/if}
  {#if form?.success}
    <div class="alert success">Compression successful.</div>
  {/if}

  <form method="POST" class="compress-form">
    <label>
      Prompt
      <textarea
        name="text"
        rows="10"
        required
        placeholder="Enter the prompt text to compress"
      >{payload.text}</textarea>
    </label>

    <label>
      Tokenizer
      <select name="tokenizer_id" required>
        <option value="" disabled selected={!payload.tokenizer_id}>
          Select a tokenizer
        </option>
        {#each tokenizerOptions as option}
          <option
            value={option.value}
            selected={option.value === payload.tokenizer_id}
          >
            {option.label}
          </option>
        {/each}
      </select>
    </label>

    <fieldset>
      <legend>Rules</legend>
      <label>
        <input
          type="checkbox"
          name="rules.units"
          checked={payload.rules.units}
        />
        Preserve measurement units
      </label>
      <label>
        <input
          type="checkbox"
          name="rules.numbers"
          checked={payload.rules.numbers}
        />
        Preserve numbers
      </label>
      <label>
        <input
          type="checkbox"
          name="rules.whitespace"
          checked={payload.rules.whitespace}
        />
        Normalize whitespace
      </label>
    </fieldset>

    <fieldset>
      <legend>Policy</legend>
      <label>
        <input
          type="checkbox"
          name="policy.protect_code"
          checked={payload.policy.protect_code}
        />
        Protect code blocks
      </label>
      <label>
        <input
          type="checkbox"
          name="policy.protect_json_keys"
          checked={payload.policy.protect_json_keys}
        />
        Protect JSON keys
      </label>
      <label>
        Minimum savings (%)
        <input
          type="number"
          name="policy.min_saving_pct"
          min="0"
          max="100"
          step="1"
          value={payload.policy.min_saving_pct}
        />
      </label>
    </fieldset>

    <button type="submit">Compress prompt</button>
  </form>

  {#if form?.response}
    <section class="results">
      <h2>Results</h2>
      <div class="stats">
        <div>
          <span class="label">Original tokens</span>
          <span class="value">{form.response.original_tokens ?? '—'}</span>
        </div>
        <div>
          <span class="label">Compressed tokens</span>
          <span class="value">{form.response.compressed_tokens ?? '—'}</span>
        </div>
        <div>
          <span class="label">Savings (%)</span>
          <span class="value">{form.response.savings_pct ?? '—'}</span>
        </div>
      </div>
      <h3>Compressed prompt</h3>
      <pre>{form.response.compressed_text ?? 'No text returned.'}</pre>
    </section>
  {/if}
</main>

<style>
  :global(body) {
    font-family: system-ui, sans-serif;
    background: #f7f7f8;
    margin: 0;
    color: #1f2933;
  }

  .container {
    max-width: 960px;
    margin: 2rem auto;
    padding: 2rem;
    background: white;
    border-radius: 1rem;
    box-shadow: 0 1.5rem 3rem -2rem rgba(15, 23, 42, 0.5);
  }

  h1 {
    margin-top: 0;
  }

  .description {
    margin-bottom: 2rem;
    color: #4b5563;
  }

  form.compress-form {
    display: grid;
    gap: 1.5rem;
  }

  textarea,
  select,
  input[type='number'] {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #d1d5db;
    border-radius: 0.5rem;
    font-size: 1rem;
    font-family: inherit;
  }

  fieldset {
    border: 1px solid #e5e7eb;
    border-radius: 0.75rem;
    padding: 1rem 1.25rem;
  }

  fieldset legend {
    padding: 0 0.5rem;
    font-weight: 600;
  }

  fieldset label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }

  button[type='submit'] {
    background: #2563eb;
    color: white;
    border: none;
    padding: 0.9rem 1.5rem;
    font-size: 1rem;
    border-radius: 0.75rem;
    cursor: pointer;
    justify-self: start;
  }

  button[type='submit']:hover {
    background: #1d4ed8;
  }

  .alert {
    padding: 0.75rem 1rem;
    border-radius: 0.75rem;
    margin-bottom: 1rem;
  }

  .alert.error {
    background: #fee2e2;
    color: #991b1b;
  }

  .alert.success {
    background: #dcfce7;
    color: #166534;
  }

  .results {
    margin-top: 2rem;
    border-top: 1px solid #e5e7eb;
    padding-top: 1.5rem;
  }

  .stats {
    display: grid;
    gap: 1rem;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    margin-bottom: 1.5rem;
  }

  .stats .label {
    display: block;
    font-size: 0.85rem;
    color: #6b7280;
  }

  .stats .value {
    font-size: 1.25rem;
    font-weight: 600;
  }

  pre {
    background: #0f172a;
    color: #f8fafc;
    padding: 1rem;
    border-radius: 0.75rem;
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
