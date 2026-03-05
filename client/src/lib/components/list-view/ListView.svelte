<script lang="ts">
	import { Folder, Document, OverflowMenuVertical, TrashCan, Undo, Download } from 'carbon-icons-svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { fileService } from '$lib';
	import ImagePreview from '../image-preview/ImagePreview.svelte';

	let {
		files = [],
		currentPath = '.',
		onRefresh = () => {},
		isTrashMode = false,
		onRestore = undefined
	} = $props();

	let openMenuId = $state<string | null>(null);
	let previewOpen = $state(false);
	let previewImageUrl = $state('');
	let previewImageName = $state('');
	let previewFile = $state<any>(null);

	const isImageFile = (filename: string): boolean => {
		const imageExtensions = ['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'svg'];
		const extension = filename.split('.').pop()?.toLowerCase();
		return extension ? imageExtensions.includes(extension) : false;
	};

	const onFileClick = async (file: any) => {
		if (file.type === 'folder') {
			// Navigate using the folder's ID
			await goto(resolve(`/folder/${file.id}`));
		} else if (isImageFile(file.name)) {
			// Show image preview
			await handleImagePreview(file);
		} else {
			// Download file on click
			await handleDownload(file);
		}
	};

	const toggleMenu = (fileName: string, event: MouseEvent) => {
		event.stopPropagation();
		openMenuId = openMenuId === fileName ? null : fileName;
	};

	const constructFilePath = (fileName: string) => {
		if (currentPath === '.' || currentPath === '') {
			return fileName;
		}
		return `${currentPath}/${fileName}`;
	};

	const handleDelete = async (file: any, event: MouseEvent) => {
		event.stopPropagation();

		// Use the path from the file object if available, otherwise construct it
		const filePath = file.path || constructFilePath(file.name);
		const confirmMsg = `Are you sure you want to move "${file.name}" to trash?`;

		if (!confirm(confirmMsg)) {
			return;
		}

		try {
			await fileService.deleteFile({ file_path: filePath });
			openMenuId = null;
			onRefresh();
		} catch (error) {
			console.error('Failed to delete file:', error);
			alert('Failed to move file to trash');
		}
	};

	const handleRestore = async (file: any, event: MouseEvent) => {
		event.stopPropagation();

		if (onRestore) {
			try {
				await onRestore(file.id);
				openMenuId = null;
			} catch (error) {
				console.error('Failed to restore file:', error);
				alert('Failed to restore file');
			}
		}
	};

	const handleImagePreview = async (file: any) => {
		try {
			const filePath = file.path || constructFilePath(file.name);
			const blob = await fileService.downloadFile({ file_path: filePath });
			
			// Create object URL for preview
			const url = window.URL.createObjectURL(blob);
			previewImageUrl = url;
			previewImageName = file.name;
			previewFile = file;
			previewOpen = true;
		} catch (error) {
			console.error('Failed to load image preview:', error);
			alert('Failed to load image preview');
		}
	};

	const handlePreviewDownload = async () => {
		if (previewFile) {
			await handleDownload(previewFile);
		}
		// Clean up the object URL
		if (previewImageUrl) {
			window.URL.revokeObjectURL(previewImageUrl);
		}
	};

	const handleDownload = async (file: any, event?: MouseEvent) => {
		if (event) {
			event.stopPropagation();
		}

		// Only download files, not folders
		if (file.type === 'folder') {
			return;
		}

		try {
			const filePath = file.path || constructFilePath(file.name);
			const blob = await fileService.downloadFile({ file_path: filePath });

			// Create download link
			const url = window.URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = file.name;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			window.URL.revokeObjectURL(url);

			if (event) {
				openMenuId = null;
			}
		} catch (error) {
			console.error('Failed to download file:', error);
			alert('Failed to download file');
		}
	};

	// Close menu when clicking outside
	const handleClickOutside = () => {
		openMenuId = null;
	};
</script>

{#snippet file_render(file: any)}
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
			<button
				class="action-button"
				aria-label="More actions"
				onclick={(e) => toggleMenu(file.name, e)}
			>
				<OverflowMenuVertical size={20} />
			</button>

			{#if openMenuId === file.name}
				<div class="action-menu">
					{#if isTrashMode}
						<button class="menu-item restore-item" onclick={(e) => handleRestore(file, e)}>
							<Undo size={16} />
							<span>Restore</span>
						</button>
					{:else}
						{#if file.type !== 'folder'}
							<button class="menu-item" onclick={(e) => handleDownload(file, e)}>
								<Download size={16} />
								<span>Download</span>
							</button>
						{/if}
						<button class="menu-item delete-item" onclick={(e) => handleDelete(file, e)}>
							<TrashCan size={16} />
							<span>Move to trash</span>
						</button>
					{/if}
				</div>
			{/if}
		</div>
	</div>
{/snippet}

<svelte:window onclick={handleClickOutside} />

<div class="file-list">
	<div class="file-list-header">
		<div class="file-name">Name</div>
		<div class="file-modified">Last modified</div>
		<div class="file-size">File size</div>
		<div class="file-actions"></div>
	</div>

	{#each files as file (file.name)}
		{@render file_render(file)}
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
		position: relative;
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

	.action-menu {
		position: absolute;
		top: 100%;
		right: 0;
		background: white;
		border: 1px solid #dadce0;
		border-radius: 8px;
		box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
		z-index: 1000;
		min-width: 180px;
		margin-top: 4px;
	}

	.menu-item {
		display: flex;
		align-items: center;
		gap: 12px;
		width: 100%;
		padding: 10px 16px;
		background: none;
		border: none;
		cursor: pointer;
		font-size: 14px;
		color: #202124;
		text-align: left;
		transition: background-color 0.15s;
	}

	.menu-item:hover {
		background-color: #f1f3f4;
	}

	.menu-item:first-child {
		border-radius: 8px 8px 0 0;
	}

	.menu-item:last-child {
		border-radius: 0 0 8px 8px;
	}

	.delete-item {
		color: #d93025;
	}

	.delete-item:hover {
		background-color: #fce8e6;
	}

	.restore-item {
		color: #1a73e8;
	}

	.restore-item:hover {
		background-color: #e8f0fe;
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

<ImagePreview
	bind:isOpen={previewOpen}
	imageUrl={previewImageUrl}
	imageName={previewImageName}
	onDownload={handlePreviewDownload}
/>
