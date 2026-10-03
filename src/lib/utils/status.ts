export type Tone = 'neutral' | 'accent' | 'success' | 'warning' | 'danger';

const tones: Record<string, Tone> = {
	pending: 'accent',
	running: 'accent',
	schema_review: 'accent',
	paused: 'warning',
	completed: 'success',
	failed: 'danger',
	cancelled: 'neutral',
};

export function statusTone(status: string): Tone {
	return tones[status] ?? 'neutral';
}

export function statusLabel(status: string): string {
	const text = status.replaceAll('_', ' ');
	return text.charAt(0).toUpperCase() + text.slice(1);
}
