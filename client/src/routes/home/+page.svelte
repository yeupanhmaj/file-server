<script lang="ts">
	import { fileService, GridView, ListView } from '$lib';
	import FolderViewContainer from '$lib/components/folder-view-container/FolderViewContainer.svelte';

	let viewMode = $state<'grid' | 'list'>('list');

	const toggleView = () => {
		if (viewMode === 'list') {
			viewMode = 'grid';
		} else {
			viewMode = 'list';
		}
	};

	let filesPromise = $state(fileService.getListFileAndFolder({ path: '.' }));
</script>

<FolderViewContainer viewName="My Drive" {toggleView} {viewMode}>
	{#await filesPromise}
		<p>Loading files...</p>
	{:then files}
		{#if viewMode === 'list'}
			<ListView {files} />
		{:else}
			<GridView {files} />
		{/if}
	{:catch error}
		<p>Error loading files: {error.message}</p>
	{/await}
</FolderViewContainer>
