<script lang="ts">
	import { Grid, List } from 'carbon-icons-svelte';

	let { children, viewName, toggleView, viewMode } = $props();
</script>

<div class="container">
	<div class="toolbar">
		<h2 class="page-title">{viewName}</h2>
		<div class="toolbar-right">
			<button
				class="view-toggle"
				onclick={toggleView}
				aria-label="Toggle view"
				title="View: {viewMode}"
			>
				{#if viewMode === 'list'}
					<Grid size={20} />
				{:else}
					<List size={20} />
				{/if}
			</button>
		</div>
	</div>

	{@render children()}
</div>

<style>
	.container {
		max-width: 1400px;
		margin: 0 auto;
	}

	.toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 24px;
		padding-bottom: 16px;
		border-bottom: 1px solid #e0e0e0;
	}

	.page-title {
		font-size: 24px;
		font-weight: 400;
		color: #202124;
		margin: 0;
	}

	.toolbar-right {
		display: flex;
		gap: 8px;
	}

	.view-toggle {
		background: none;
		border: none;
		cursor: pointer;
		padding: 8px;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #5f6368;
		transition: background-color 0.2s;
	}
</style>
