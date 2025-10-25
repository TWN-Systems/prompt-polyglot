import { env } from '$env/dynamic/public';

export const API_BASE = env.PUBLIC_API_BASE ?? 'http://localhost:8080';
