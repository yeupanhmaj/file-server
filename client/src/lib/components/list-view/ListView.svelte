<script lang="ts">
	import { Folder, Document, OverflowMenuVertical } from 'carbon-icons-svelte';

	let { files = [] } = $props();

	const onFileClick = (file: any) => {
		console.log('Clicked file:', file);
	};
</script>

<div class="file-list">
	<div class="file-list-header">
		<div class="file-name">Name</div>
		<div class="file-modified">Last modified</div>
		<div class="file-size">File size</div>
		<div class="file-actions"></div>
	</div>

	{#each files as file}
		<div
			role="button"
			tabindex="0"
			class="file-row"
			onclick={() => onFileClick(file)}
			onkeydown={(e) => {
				if (e.key === 'Enter' || e.key === ' ') {
					e.preventDefault();
					onFileClick(file);
				}
			}}
		>
			<div class="file-name">
				{#if file.type === 'folder'}
					<Folder size={20} />
				{:else}
					<Document size={20} />
				{/if}
				<span>{file.name}</span>
			</div>
			<div class="file-modified">{file.modified}</div>
			<div class="file-size">{file.size}</div>
			<div class="file-actions">
				<button class="action-button" aria-label="More actions">
					<OverflowMenuVertical size={20} />
				</button>
			</div>
		</div>
	{/each}
</div>

<style>
	.file-list {
		display: flex;
		flex-direction: column;
	}

	.file-list-header,
	.file-row {
		display: grid;
		grid-template-columns: 1fr 200px 120px 48px;
		align-items: center;
		gap: 16px;
		padding: 12px 16px;
	}

	.file-list-header {
		border-bottom: 1px solid #e0e0e0;
		font-size: 12px;
		font-weight: 500;
		color: #5f6368;
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.file-row {
		border-bottom: 1px solid #f1f3f4;
		border-radius: 8px;
		transition: background-color 0.2s;
		cursor: pointer;
	}

	.file-row:hover {
		background-color: #f8f9fa;
	}

	.file-name {
		display: flex;
		align-items: center;
		gap: 12px;
		color: #202124;
		font-size: 14px;
	}

	.file-modified,
	.file-size {
		color: #5f6368;
		font-size: 13px;
	}

	.file-actions {
		display: flex;
		justify-content: flex-end;
	}

	.action-button {
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

	.file-row:hover .action-button {
		opacity: 1;
	}

	.action-button:hover {
		background-color: #e8eaed;
	}

	@media (max-width: 768px) {
		.file-list-header,
		.file-row {
			grid-template-columns: 1fr 48px;
		}

		.file-modified,
		.file-size {
			display: none;
		}
	}
</style>
