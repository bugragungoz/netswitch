/** Turns an error from the Rust side into a sentence for the user. */
import { toAppError } from '../api/invoke';
import { t } from '../i18n';

const KNOWN = new Set(['firewall_failed', 'backup_failed', 'protected_program', 'unsupported', 'not_found', 'no_documents_folder', 'settings_not_saved']);

export function errorMessage(e: unknown): string {
  const err = toAppError(e);
  const code = KNOWN.has(err.code) ? err.code : 'internal';
  return t(`errors.${code}`, { detail: err.detail || err.message });
}
