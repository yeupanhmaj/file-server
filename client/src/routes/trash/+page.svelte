<script lang="ts">
	import { fileService, GridView, ListView } from '$lib';
	import FolderViewContainer from '$lib/components/folder-view-container/FolderViewContainer.svelte';
	import type { TrashItem } from '$lib/services/types';

	let viewMode = $state<'grid' | 'list'>('list');
	let trashPromise = $state(loadTrash());
	let isEmptying = $state(false);

	async function loadTrash() {
		return fileService.listTrash();
	}

	function refreshTrash() {
		trashPromise = loadTrash();
	}

	async function handleRestore(itemId: string) {
		try {
			await fileService.restoreFile({ trash_item_id: itemId });
			refreshTrash();
		} catch (error) {
			console.error('Failed to restore file:', error);
			alert('Failed to restore file');
		}
	}

	async function handleEmptyTrash() {
		if (!confirm('Are you sure you want to permanently delete all items in trash?')) {
			return;
		}

		isEmptying = true;
		try {
			await fileService.emptyTrash();
			refreshTrash();
		} catch (error) {
			console.error('Failed to empty trash:', error);
			alert('Failed to empty trash');
		} finally {
			isEmptying = false;
		}
	}

	const toggleView = () => {
		if (viewMode === 'list') {
			viewMode = 'grid';
		} else {
			viewMode = 'list';
		}
	};

	// Convert TrashItem to file format expected by ListView/GridView
	function mapTrashToFiles(trashItems: TrashItem[]) {
		return trashItems.map((item) => ({
			id: item.id,
			name: item.name,
			type: 'file',
			modified: new Date(item.deleted_at).toLocaleDateString(),
			size: item.size,
			originalPath: item.original_path
		}));
	}
</script>

<FolderViewContainer viewName="Trash" {toggleView} {viewMode}>
	{#await trashPromise}
		<p>Loading trash...</p>
	{:then trashItems}
		{#if trashItems.length === 0}
			<div class="empty-state">
				<p>Trash is empty</p>
			</div>
		{:else}
			<div class="trash-actions">
				<button onclick={handleEmptyTrash} disabled={isEmptying}>
					{isEmptying ? 'Emptying...' : 'Empty Trash'}
				</button>
			</div>
			{@const files = mapTrashToFiles(trashItems)}
			{#if viewMode === 'list'}
				<ListView {files} />
			{:else}
				<GridView {files} />
			{/if}
		{/if}
	{:catch error}
		<p>Error loading trash: {error.message}</p>
	{/await}
</FolderViewContainer>

<style>
	.empty-state {
		display: flex;
		justify-content: center;
		align-items: center;
		min-height: 300px;
		color: #666;
		font-size: 1.1rem;
	}

	.trash-actions {
		display: flex;
		justify-content: flex-end;
		padding: 1rem;
		gap: 0.5rem;
	}

	.trash-actions button {
		padding: 0.5rem 1rem;
		background-color: #dc3545;
		color: white;
		border: none;
		border-radius: 4px;
		cursor: pointer;
		font-size: 0.9rem;
	}

	.trash-actions button:hover:not(:disabled) {
		background-color: #c82333;
	}

	.trash-actions button:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}
</style>
