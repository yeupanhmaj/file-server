<script lang="ts">
	import { page } from '$app/state';

	import { fileService, FolderViewContainer, ListView, GridView } from '$lib';

	let viewMode = $state<'grid' | 'list'>('list');

	const toggleView = () => {
		if (viewMode === 'list') {
			viewMode = 'grid';
		} else {
			viewMode = 'list';
		}
	};

	// The slug is now the folder ID
	const folderId = $derived(page.params.slug || '');

	// Folder name will be determined from the files response
	let folderName = $state('Loading...');

	const loadFiles = async () => {
		const files = await fileService.getFolderById({ folder_id: folderId });
		// Extract folder name from the first file's parent path (if available)
		if (files.length > 0 && files[0].path) {
			const pathParts = files[0].path.split('/');
			if (pathParts.length > 1) {
				folderName = pathParts[pathParts.length - 2];
			} else {
				folderName = 'Root';
			}
		} else {
			folderName = 'Folder';
		}
		return files;
	};

	// Reload files when folder ID changes
	let filesPromise = $derived(loadFiles());

	const refreshFiles = () => {
		// Force a refresh by updating currentPath or refetching
		filesPromise = loadFiles();
	};
</script>

<FolderViewContainer viewName={folderName} {toggleView} {viewMode}>
	{#await filesPromise}
		<p>Loading files...</p>
	{:then files}
		{#if viewMode === 'list'}
			<ListView {files} currentPath={folderId} onRefresh={refreshFiles} />
		{:else}
			<GridView {files} currentPath={folderId} onRefresh={refreshFiles} />
		{/if}
	{:catch error}
		<p>Error loading files: {error.message}</p>
	{/await}
</FolderViewContainer>
