<script lang="ts">
	import { fileService, GridView, ListView } from '$lib';
	import FolderViewContainer from '$lib/components/folder-view-container/FolderViewContainer.svelte';

	let viewMode = $state<'grid' | 'list'>('list');
	let currentPath = '.';

	const toggleView = () => {
		if (viewMode === 'list') {
			viewMode = 'grid';
		} else {
			viewMode = 'list';
		}
	};

	const loadFiles = () => fileService.getListFileAndFolder({ path: currentPath });

	let filesPromise = $state(loadFiles());

	const refreshFiles = () => {
		filesPromise = loadFiles();
	};
</script>

<FolderViewContainer viewName="My Drive" {toggleView} {viewMode}>
	{#await filesPromise}
		<p>Loading files...</p>
	{:then files}
		{#if viewMode === 'list'}
			<ListView {files} {currentPath} onRefresh={refreshFiles} />
		{:else}
			<GridView {files} {currentPath} onRefresh={refreshFiles} />
		{/if}
	{:catch error}
		<p>Error loading files: {error.message}</p>
	{/await}
</FolderViewContainer>
