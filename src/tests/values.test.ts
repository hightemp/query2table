import { describe, it, expect } from 'vitest';
import {
	columnLabel,
	formatCell,
	isMissing,
	parseNumber,
	parseBoolean,
	sortValue,
} from '$lib/utils/values';

describe('cell values', () => {
	it('labels machine-style column names readably', () => {
		expect(columnLabel('ceo_name')).toBe('CEO name');
		expect(columnLabel('funding_amount')).toBe('Funding amount');
		expect(columnLabel('websiteUrl')).toBe('Website URL');
		expect(columnLabel('Count')).toBe('Count');
		expect(columnLabel('Company Name')).toBe('Company Name');
	});

	it('treats placeholder text as missing information', () => {
		for (const value of [null, undefined, '', '  ', 'Unknown', 'N/A', 'not found', []])
			expect(isMissing(value)).toBe(true);
		expect(isMissing(0)).toBe(false);
		expect(isMissing(false)).toBe(false);
	});

	it('formats numbers, booleans and dates by column type', () => {
		expect(parseNumber('$1,200.5')).toBe(1200.5);
		expect(parseNumber('12 employees')).toBeNull();
		expect(formatCell(500000000, 'number')).toBe((500000000).toLocaleString());
		expect(formatCell('45%', 'number')).toBe('45%');
		expect(formatCell('12 employees', 'number')).toBe('12 employees');
		expect(parseBoolean('Yes')).toBe(true);
		expect(formatCell(false, 'boolean')).toBe('No');
		expect(formatCell('2024-03-05', 'date')).toBe(
			new Date('2024-03-05').toLocaleDateString(undefined, {
				timeZone: 'UTC',
				year: 'numeric',
				month: 'short',
				day: 'numeric',
			})
		);
		expect(formatCell('Spring 2024', 'date')).toBe('Spring 2024');
		expect(formatCell('Unknown', 'number')).toBe('Unknown');
		expect(formatCell(null, 'text')).toBe('—');
	});

	it('sorts numbers numerically and leaves missing values for last', () => {
		expect(sortValue('1,000', 'number')).toBe(1000);
		expect(sortValue('Unknown', 'number')).toBeNull();
		expect(sortValue('2024-01-02', 'date')).toBe(Date.parse('2024-01-02'));
		expect(sortValue(true, 'boolean')).toBe(1);
	});
});

describe('column headers in other languages', () => {
	it('are shown as written', async () => {
		const { columnLabel } = await import('$lib/utils/values');
		expect(columnLabel('Число сотрудников')).toBe('Число сотрудников');
		expect(columnLabel('Сайт')).toBe('Сайт');
		expect(columnLabel('название_компании')).toBe('Название компании');
	});
});
