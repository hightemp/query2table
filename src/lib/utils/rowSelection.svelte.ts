/** The row open in the detail panel and the displayed row order used to step through rows. */
export class RowSelection {
	id = $state<string | null>(null);
	order = $state<string[]>([]);

	position = $derived.by(() => {
		const index = this.id === null ? -1 : this.order.indexOf(this.id);
		return index < 0 ? null : { index, total: this.order.length };
	});

	select = (id: string, order: string[]) => {
		this.id = id;
		this.order = order;
	};

	move = (delta: number) => {
		const next = this.position ? this.order[this.position.index + delta] : undefined;
		if (next !== undefined) this.id = next;
	};

	clear = () => {
		this.id = null;
	};
}
