import { fail, type Actions } from '@sveltejs/kit';
import { API_BASE } from '$lib/config';
import type { CompressRequest, CompressResponse } from '$lib/types';

export interface ActionResult {
  success?: boolean;
  error?: string;
  payload?: CompressRequest;
  response?: CompressResponse;
}

export const actions: Actions = {
  default: async ({ request, fetch }) => {
    const formData = await request.formData();

    const payload: CompressRequest = {
      text: String(formData.get('text') ?? ''),
      tokenizer_id: String(formData.get('tokenizer_id') ?? ''),
      rules: {
        units: formData.has('rules.units'),
        numbers: formData.has('rules.numbers'),
        whitespace: formData.has('rules.whitespace')
      },
      policy: {
        protect_code: formData.has('policy.protect_code'),
        protect_json_keys: formData.has('policy.protect_json_keys'),
        min_saving_pct: Number(formData.get('policy.min_saving_pct') ?? 0)
      }
    };

    if (!payload.text.trim()) {
      return fail(400, { error: 'Prompt text is required.', payload });
    }

    if (!payload.tokenizer_id) {
      return fail(400, { error: 'Tokenizer is required.', payload });
    }

    try {
      const response = await fetch(`${API_BASE}/compress`, {
        method: 'POST',
        headers: {
          'content-type': 'application/json'
        },
        body: JSON.stringify(payload)
      });

      if (!response.ok) {
        const message = await response.text();
        return fail(response.status, {
          error: message || 'Compression request failed.',
          payload
        });
      }

      const data = (await response.json()) as CompressResponse;

      return {
        success: true,
        payload,
        response: data
      } satisfies ActionResult;
    } catch (error) {
      console.error('Failed to call compression API', error);
      return fail(500, {
        error: 'Unable to reach compression service.',
        payload
      });
    }
  }
};
