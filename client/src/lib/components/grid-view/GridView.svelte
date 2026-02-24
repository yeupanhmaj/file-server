<script lang="ts">
	import { Folder, Document, OverflowMenuVertical } from 'carbon-icons-svelte';

	let { files = [], currentPath = '.', onRefresh = () => {} } = $props();
</script>

<div class="file-grid">
	{#each files as file}
		<div class="file-card">
			<div class="file-icon">
				{#if file.type === 'folder'}
					<Folder size={32} />
				{:else}
					<Document size={32} />
				{/if}
			</div>
			<div class="file-info">
				<div class="file-card-name">{file.name}</div>
				<button class="file-card-menu" aria-label="More actions">
					<OverflowMenuVertical size={20} />
				</button>
			</div>
		</div>
	{/each}
</div>

<style>
	.file-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
		gap: 16px;
	}

	.file-card {
		border: 1px solid #e0e0e0;
		border-radius: 8px;
		padding: 16px;
		cursor: pointer;
		transition: all 0.2s;
	}

	.file-card:hover {
		background-color: #f8f9fa;
		box-shadow: 0 1px 3px rgba(60, 64, 67, 0.3);
	}

	.file-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 80px;
		color: #5f6368;
	}

	.file-info {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-top: 8px;
	}

	.file-card-name {
		font-size: 14px;
		color: #202124;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		flex: 1;
	}

	.file-card-menu {
		background: none;
		border: none;
		cursor: pointer;
		padding: 4px;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #5f6368;
		opacity: 0;
		transition: all 0.2s;
	}

	.file-card:hover .file-card-menu {
		opacity: 1;
	}

	.file-card-menu:hover {
		background-color: #e8eaed;
	}

	@media (max-width: 768px) {
		.file-grid {
			grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
		}
	}
</style>
