/**
 * Shared formatting utilities
 * Extracted to avoid code duplication across components
 */

/**
 * Get Material color for confidence level chip
 */
export function getConfidenceColor(level: string): string {
  switch (level) {
    case 'established':
      return 'primary';
    case 'emerging':
      return 'accent';
    case 'provisional':
      return 'warn';
    default:
      return '';
  }
}

/**
 * Format date string to DD/MM/YYYY format
 */
export function formatDate(dateString: string | null | undefined): string {
  if (!dateString) return 'N/A';
  const date = new Date(dateString);
  const day = date.getDate().toString().padStart(2, '0');
  const month = (date.getMonth() + 1).toString().padStart(2, '0');
  const year = date.getFullYear();
  return `${day}/${month}/${year}`;
}

/**
 * Format probability as percentage string
 */
export function formatProbability(prob: number): string {
  return (prob * 100).toFixed(1) + '%';
}
