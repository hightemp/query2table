/** Stand-in for `$app/state` in component tests: set `page.params` to change the route. */
export const page = $state<{ params: Record<string, string> }>({ params: {} });
