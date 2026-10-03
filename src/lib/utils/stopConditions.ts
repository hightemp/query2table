import type { StopConditions } from '$lib/api/tauri';

export interface StopConditionInput {
	target: string;
	budget: string;
	/** Minutes */
	duration: string;
}

export type StopConditionErrors = Partial<Record<keyof StopConditionInput, string>>;

/** Settings keys that store the last used stop conditions. */
export const STOP_SETTING_KEYS = {
	target: 'target_row_count',
	budget: 'max_budget_usd',
	duration: 'max_duration_seconds',
} as const;

export const DEFAULT_STOP_INPUT: StopConditionInput = { target: '50', budget: '1.00', duration: '10' };

const MAX_TARGET = 10_000;
const MAX_MINUTES = 24 * 60;

export function stopInputFromSettings(settings: Map<string, string>): StopConditionInput {
	const target = Number(settings.get(STOP_SETTING_KEYS.target));
	const budget = Number(settings.get(STOP_SETTING_KEYS.budget));
	const seconds = Number(settings.get(STOP_SETTING_KEYS.duration));
	return {
		target: Number.isInteger(target) && target > 0 ? String(target) : DEFAULT_STOP_INPUT.target,
		budget: Number.isFinite(budget) && budget > 0 ? budget.toFixed(2) : DEFAULT_STOP_INPUT.budget,
		duration:
			Number.isFinite(seconds) && seconds > 0
				? String(Math.max(1, Math.round(seconds / 60)))
				: DEFAULT_STOP_INPUT.duration,
	};
}

export function parseStopConditions(input: StopConditionInput): {
	conditions: Required<StopConditions> | null;
	errors: StopConditionErrors;
} {
	const errors: StopConditionErrors = {};
	const target = Number(input.target.trim());
	if (!input.target.trim() || !Number.isInteger(target) || target < 1 || target > MAX_TARGET)
		errors.target = `Enter a whole number from 1 to ${MAX_TARGET.toLocaleString('en-US')}.`;
	const budget = Number(input.budget.trim());
	if (!input.budget.trim() || !Number.isFinite(budget) || budget < 0.01)
		errors.budget = 'Enter an amount of at least $0.01.';
	const minutes = Number(input.duration.trim());
	if (!input.duration.trim() || !Number.isFinite(minutes) || minutes < 1 || minutes > MAX_MINUTES)
		errors.duration = `Enter from 1 to ${MAX_MINUTES} minutes.`;
	if (Object.keys(errors).length) return { conditions: null, errors };
	return {
		conditions: {
			target_row_count: target,
			max_budget_usd: budget,
			max_duration_seconds: Math.round(minutes * 60),
		},
		errors,
	};
}

export function formatUsd(amount: number): string {
	const cents = amount * 100;
	return Math.abs(cents - Math.round(cents)) < 1e-9 ? `$${amount.toFixed(2)}` : `$${amount}`;
}

export function formatMinutes(seconds: number): string {
	const minutes = Math.round(seconds / 60);
	return minutes >= 60 && minutes % 60 === 0 ? `${minutes / 60} h` : `${minutes} min`;
}
