<script lang="ts">
	import {
		Folder,
		Document,
		OverflowMenuVertical,
		TrashCan,
		Undo,
		Download
	} from 'carbon-icons-svelte';
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

	const toggleMenu = (fileId: string, event: MouseEvent) => {
		event.stopPropagation();
		openMenuId = openMenuId === fileId ? null : fileId;
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

<svelte:window onclick={handleClickOutside} />

<div class="file-grid">
	{#each files as file (file.id || file.name)}
		<div
			class="file-card"
			role="button"
			tabindex="0"
			onclick={() => onFileClick(file)}
			onkeydown={(e) => {
				if (e.key === 'Enter' || e.key === ' ') {
					e.preventDefault();
					onFileClick(file);
				}
			}}
		>
			<div class="file-icon">
				{#if file.type === 'folder'}
					<Folder size={32} />
				{:else}
					<Document size={32} />
				{/if}
			</div>
			<div class="file-info">
				<div class="file-card-name">{file.name}</div>
				<button
					class="file-card-menu"
					aria-label="More actions"
					onclick={(e) => toggleMenu(file.id || file.name, e)}
				>
					<OverflowMenuVertical size={20} />
				</button>

				{#if openMenuId === (file.id || file.name)}
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
	{/each}
</div>
<ImagePreview
	bind:isOpen={previewOpen}
	imageUrl={previewImageUrl}
	imageName={previewImageName}
	onDownload={handlePreviewDownload}
/>

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
		position: relative;
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
		.file-grid {
			grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
		}
	}
</style>
