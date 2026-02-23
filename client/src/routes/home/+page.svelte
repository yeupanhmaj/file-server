<script lang="ts">
	import { GridView, ListView } from '$lib';
	import FolderViewContainer from '$lib/components/folder-view-container/FolderViewContainer.svelte';

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

<FolderViewContainer viewName="My Drive" {toggleView} {viewMode}>
	{#if viewMode === 'list'}
		<ListView {files} />
	{:else}
		<GridView {files} />
	{/if}
</FolderViewContainer>
