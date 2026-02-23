<script lang="ts">
	import { Grid, List } from 'carbon-icons-svelte';
	import ListView from '$lib/components/ListView.svelte';
	import GridView from '$lib/components/GridView.svelte';

	let viewMode = $state<'grid' | 'list'>('list');

	// Sample hierarchical data for tree view

	// Sample flat data for list/grid views
	const files = [
		{ id: 1, name: 'Documents', type: 'folder', modified: '2024-02-20', size: '-' },
		{ id: 2, name: 'Images', type: 'folder', modified: '2024-02-19', size: '-' },
		{ id: 3, name: 'Project Report.pdf', type: 'file', modified: '2024-02-18', size: '2.4 MB' },
		{
			id: 4,
			name: 'Meeting Notes.txt',
			type: 'file',
			modified: '2024-02-15',
			size: '12 KB'
		},
		{ id: 5, name: 'Presentation.pptx', type: 'file', modified: '2024-02-10', size: '5.8 MB' }
	];

	const toggleView = () => {
		if (viewMode === 'list') {
			viewMode = 'grid';
		} else {
			viewMode = 'list';
		}
	};
</script>

<div class="container">
	<div class="toolbar">
		<h2 class="page-title">My Drive</h2>
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

	{#if viewMode === 'list'}
		<ListView {files} />
	{:else}
		<GridView {files} />
	{/if}
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
