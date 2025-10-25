export interface CompressionRules {
  units: boolean;
  numbers: boolean;
  whitespace: boolean;
}

export interface CompressionPolicy {
  protect_code: boolean;
  protect_json_keys: boolean;
  min_saving_pct: number;
}

export interface CompressRequest {
  text: string;
  tokenizer_id: string;
  rules: CompressionRules;
  policy: CompressionPolicy;
}

export interface CompressResponse {
  compressed_text: string;
  original_tokens?: number;
  compressed_tokens?: number;
  savings_pct?: number;
  [key: string]: unknown;
}
