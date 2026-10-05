import { t } from '$lib/i18n';
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

/** Research counts agent steps, so its limit is remembered separately from result counts. */
export function stopSettingKeys(mode: string) {
	return mode === 'research'
		? { ...STOP_SETTING_KEYS, target: 'research_max_steps' as const }
		: STOP_SETTING_KEYS;
}

const RESEARCH_DEFAULT_STEPS = '100';
const RESEARCH_MAX_STEPS = 200;

export const DEFAULT_STOP_INPUT: StopConditionInput = { target: '50', budget: '1.00', duration: '10' };

const MAX_TARGET = 10_000;
const MAX_MINUTES = 24 * 60;

export function stopInputFromSettings(
	settings: Map<string, string>,
	mode = 'table'
): StopConditionInput {
	const keys = stopSettingKeys(mode);
	const target = Number(settings.get(keys.target));
	const defaultTarget = mode === 'research' ? RESEARCH_DEFAULT_STEPS : DEFAULT_STOP_INPUT.target;
	const budget = Number(settings.get(STOP_SETTING_KEYS.budget));
	const seconds = Number(settings.get(STOP_SETTING_KEYS.duration));
	return {
		target: Number.isInteger(target) && target > 0 ? String(target) : defaultTarget,
		budget: Number.isFinite(budget) && budget > 0 ? budget.toFixed(2) : DEFAULT_STOP_INPUT.budget,
		duration:
			Number.isFinite(seconds) && seconds > 0
				? String(Math.max(1, Math.round(seconds / 60)))
				: DEFAULT_STOP_INPUT.duration,
	};
}

export function parseStopConditions(
	input: StopConditionInput,
	mode = 'table'
): {
	conditions: Required<StopConditions> | null;
	errors: StopConditionErrors;
} {
	const errors: StopConditionErrors = {};
	const maxTarget = mode === 'research' ? RESEARCH_MAX_STEPS : MAX_TARGET;
	const target = Number(input.target.trim());
	if (!input.target.trim() || !Number.isInteger(target) || target < 1 || target > maxTarget)
		errors.target = t('stop.targetError', { max: maxTarget });
	const budget = Number(input.budget.trim());
	if (!input.budget.trim() || !Number.isFinite(budget) || budget < 0.01)
		errors.budget = t('stop.budgetError');
	const minutes = Number(input.duration.trim());
	if (!input.duration.trim() || !Number.isFinite(minutes) || minutes < 1 || minutes > MAX_MINUTES)
		errors.duration = t('stop.durationError', { max: MAX_MINUTES });
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
	return minutes >= 60 && minutes % 60 === 0
		? t('units.hours', { count: minutes / 60 })
		: t('units.minutes', { count: minutes });
}
