/**
 * Formats seconds into human-readable time format
 * @param seconds - Number of seconds
 * @returns Formatted string like "2h 15m 30s" or "5m 45s" or "30s"
 */
export function formatTimeRemaining(seconds: number): string {
  if (seconds <= 0) return "0s";

  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const remainingSeconds = Math.floor(seconds % 60);

  const parts: string[] = [];

  if (hours > 0) {
    parts.push(`${hours}h`);
  }

  if (minutes > 0) {
    parts.push(`${minutes}m`);
  }

  if (remainingSeconds > 0 || parts.length === 0) {
    parts.push(`${remainingSeconds}s`);
  }

  return parts.join(' ');
}

/**
 * Formats elapsed time for completed operations
 * @param seconds - Number of seconds elapsed
 * @returns Formatted string with appropriate precision
 */
export function formatElapsedTime(seconds: number): string {
  if (seconds < 1) {
    return `${Math.round(seconds * 1000)}ms`;
  }

  if (seconds < 60) {
    return `${seconds.toFixed(1)}s`;
  }

  return formatTimeRemaining(seconds);
}
