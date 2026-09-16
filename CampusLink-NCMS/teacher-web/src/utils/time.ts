export function formatDateTime(value?: string | null): string {
  if (!value) return '-'
  const d = new Date(value)
  if (isNaN(d.getTime())) return value
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

export function isStale(lastSeenAt?: string | null, thresholdMs = 30000): boolean {
  if (!lastSeenAt) return true
  const d = new Date(lastSeenAt)
  if (isNaN(d.getTime())) return true
  return Date.now() - d.getTime() > thresholdMs
}
