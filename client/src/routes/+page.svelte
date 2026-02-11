<script lang="ts">
	const getListFile = async () => {
		const response = await fetch('http://localhost:3000/api/ls');
		return await response.json();
	};
	const ls = $state<Promise<string[]>>(getListFile());

	// Helper function to parse the entries
	function parseEntry(entry: string) {
		const match = entry.match(/\[(FILE|DIR)\] (.+)/);
		if (match) {
			return {
				type: match[1] as 'FILE' | 'DIR',
				name: match[2]
			};
		}
		return { type: 'FILE' as const, name: entry };
	}
</script>

{#await ls}
	<p>Loading...</p>
{:then values}
	{#each values as value (value)}
		{@const item = parseEntry(value)}
		<p>
			{#if item.type === 'DIR'}
				📁 {item.name}
			{:else}
				📄 {item.name}
			{/if}
		</p>
	{/each}
{:catch error}
	<p>Something went wrong: {error.message}</p>
{/await}
