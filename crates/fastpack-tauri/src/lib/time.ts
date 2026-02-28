/** Returns the current time as an HH:MM:SS string. Used for log entry timestamps. */
export function now(): string {
	const d = new Date();
	const pad = (n: number) => String(n).padStart(2, "0");
	return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}
