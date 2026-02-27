<script lang="ts">
	import { CloudUpload, Close } from 'carbon-icons-svelte';
	import { fileService } from '$lib';

	let {
		isOpen = $bindable(false),
		currentPath = '.',
		onUploadComplete = () => {}
	}: {
		isOpen: boolean;
		currentPath?: string;
		onUploadComplete?: () => void;
	} = $props();

	let fileInput = $state<HTMLInputElement>();
	let selectedFiles = $state<File[]>([]);
	let isUploading = $state(false);
	let uploadProgress = $state(0);
	let errorMessage = $state('');

	const handleFileSelect = (e: Event) => {
		const input = e.target as HTMLInputElement;
		if (input.files) {
			selectedFiles = Array.from(input.files);
			errorMessage = '';
		}
	};

	const handleDrop = (e: DragEvent) => {
		e.preventDefault();
		if (e.dataTransfer?.files) {
			selectedFiles = Array.from(e.dataTransfer.files);
			errorMessage = '';
		}
	};

	const handleDragOver = (e: DragEvent) => {
		e.preventDefault();
	};

	const removeFile = (index: number) => {
		selectedFiles = selectedFiles.filter((_, i) => i !== index);
	};

	const formatFileSize = (bytes: number): string => {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
	};

	const handleUpload = async () => {
		if (selectedFiles.length === 0) {
			errorMessage = 'Please select at least one file';
			return;
		}

		isUploading = true;
		errorMessage = '';
		uploadProgress = 0;

		try {
			const formData = new FormData();
			formData.append('path', currentPath);

			selectedFiles.forEach((file) => {
				formData.append('files', file);
			});

			await fileService.uploadFile(formData);

			uploadProgress = 100;
			setTimeout(() => {
				isOpen = false;
				selectedFiles = [];
				onUploadComplete();
			}, 500);
		} catch (error) {
			console.error('Upload failed:', error);
			errorMessage = 'Upload failed. Please try again.';
		} finally {
			isUploading = false;
		}
	};

	const closeDialog = () => {
		if (!isUploading) {
			isOpen = false;
			selectedFiles = [];
			errorMessage = '';
		}
	};

	const handleDialogKeydown = (e: KeyboardEvent) => {
		if (e.key === 'Escape' && !isUploading) {
			closeDialog();
		}
	};

	const handleDropZoneKeydown = (e: KeyboardEvent) => {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			fileInput?.click();
		}
	};
</script>

{#if isOpen}
	<div class="dialog-overlay" onclick={closeDialog} role="presentation">
		<div
			class="dialog"
			onclick={(e) => e.stopPropagation()}
			onkeydown={handleDialogKeydown}
			role="dialog"
			aria-modal="true"
			tabindex="-1"
		>
			<div class="dialog-header">
				<h2>Upload Files</h2>
				<button
					class="close-button"
					onclick={closeDialog}
					disabled={isUploading}
					aria-label="Close"
				>
					<Close size={20} />
				</button>
			</div>

			<div class="dialog-content">
				<div class="upload-info">
					<p>Uploading to: <strong>{currentPath === '.' ? 'My Drive' : currentPath}</strong></p>
				</div>

				<div
					class="drop-zone"
					ondrop={handleDrop}
					ondragover={handleDragOver}
					onclick={() => fileInput?.click()}
					onkeydown={handleDropZoneKeydown}
					role="button"
					tabindex="0"
				>
					<CloudUpload size={32} />
					<p class="drop-text">Drag files here or click to browse</p>
					<p class="drop-hint">You can select multiple files</p>
				</div>

				<input
					bind:this={fileInput}
					type="file"
					multiple
					onchange={handleFileSelect}
					style="display: none;"
				/>

				{#if selectedFiles.length > 0}
					<div class="selected-files">
						<h3>Selected Files ({selectedFiles.length})</h3>
						<div class="file-list">
							{#each selectedFiles as file, index (file.name + index)}
								<div class="file-item">
									<div class="file-info">
										<div class="file-name">{file.name}</div>
										<div class="file-size">{formatFileSize(file.size)}</div>
									</div>
									<button
										class="remove-button"
										onclick={() => removeFile(index)}
										disabled={isUploading}
										aria-label="Remove file"
									>
										<Close size={16} />
									</button>
								</div>
							{/each}
						</div>
					</div>
				{/if}

				{#if isUploading && uploadProgress > 0}
					<div class="progress-bar">
						<div class="progress-fill" style="width: {uploadProgress}%"></div>
					</div>
				{/if}

				{#if errorMessage}
					<div class="error-message">{errorMessage}</div>
				{/if}
			</div>

			<div class="dialog-footer">
				<button class="button button-secondary" onclick={closeDialog} disabled={isUploading}>
					Cancel
				</button>
				<button
					class="button button-primary"
					onclick={handleUpload}
					disabled={isUploading || selectedFiles.length === 0}
				>
					{isUploading
						? 'Uploading...'
						: `Upload ${selectedFiles.length > 0 ? `(${selectedFiles.length})` : ''}`}
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.dialog-overlay {
		position: fixed;
		inset: 0;
		background-color: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
		padding: 20px;
	}

	.dialog {
		background: white;
		border-radius: 8px;
		width: 100%;
		max-width: 600px;
		max-height: 90vh;
		display: flex;
		flex-direction: column;
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
	}

	.dialog-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 20px 24px;
		border-bottom: 1px solid #e0e0e0;
	}

	.dialog-header h2 {
		margin: 0;
		font-size: 20px;
		font-weight: 500;
		color: #202124;
	}

	.close-button {
		background: none;
		border: none;
		cursor: pointer;
		padding: 8px;
		border-radius: 50%;
		display: flex;
		align-items: center;
		color: #5f6368;
		transition: background-color 0.2s;
	}

	.close-button:hover:not(:disabled) {
		background-color: #f1f3f4;
	}

	.close-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.dialog-content {
		padding: 24px;
		overflow-y: auto;
		flex: 1;
	}

	.upload-info {
		margin-bottom: 16px;
	}

	.upload-info p {
		margin: 0;
		font-size: 14px;
		color: #5f6368;
	}

	.drop-zone {
		border: 2px dashed #dadce0;
		border-radius: 8px;
		padding: 48px 24px;
		text-align: center;
		cursor: pointer;
		transition: all 0.2s;
		background-color: #f8f9fa;
	}

	.drop-zone:hover {
		border-color: #1a73e8;
		background-color: #e8f0fe;
	}

	.drop-zone :global(svg) {
		color: #5f6368;
		margin-bottom: 16px;
	}

	.drop-text {
		margin: 0 0 8px 0;
		font-size: 16px;
		font-weight: 500;
		color: #202124;
	}

	.drop-hint {
		margin: 0;
		font-size: 14px;
		color: #5f6368;
	}

	.selected-files {
		margin-top: 24px;
	}

	.selected-files h3 {
		margin: 0 0 12px 0;
		font-size: 14px;
		font-weight: 500;
		color: #202124;
	}

	.file-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
		max-height: 200px;
		overflow-y: auto;
	}

	.file-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px;
		background-color: #f8f9fa;
		border-radius: 4px;
		gap: 12px;
	}

	.file-info {
		flex: 1;
		min-width: 0;
	}

	.file-name {
		font-size: 14px;
		color: #202124;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.file-size {
		font-size: 12px;
		color: #5f6368;
		margin-top: 4px;
	}

	.remove-button {
		background: none;
		border: none;
		cursor: pointer;
		padding: 4px;
		border-radius: 50%;
		display: flex;
		align-items: center;
		color: #5f6368;
		transition: background-color 0.2s;
		flex-shrink: 0;
	}

	.remove-button:hover:not(:disabled) {
		background-color: #e0e0e0;
	}

	.remove-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.progress-bar {
		margin-top: 16px;
		height: 4px;
		background-color: #e0e0e0;
		border-radius: 2px;
		overflow: hidden;
	}

	.progress-fill {
		height: 100%;
		background-color: #1a73e8;
		transition: width 0.3s;
	}

	.error-message {
		margin-top: 16px;
		padding: 12px;
		background-color: #fce8e6;
		color: #d93025;
		border-radius: 4px;
		font-size: 14px;
	}

	.dialog-footer {
		display: flex;
		gap: 12px;
		justify-content: flex-end;
		padding: 16px 24px;
		border-top: 1px solid #e0e0e0;
	}

	.button {
		padding: 10px 24px;
		border-radius: 4px;
		font-size: 14px;
		font-weight: 500;
		cursor: pointer;
		border: none;
		transition: all 0.2s;
	}

	.button-secondary {
		background-color: transparent;
		color: #1a73e8;
	}

	.button-secondary:hover:not(:disabled) {
		background-color: #e8f0fe;
	}

	.button-primary {
		background-color: #1a73e8;
		color: white;
	}

	.button-primary:hover:not(:disabled) {
		background-color: #1765cc;
	}

	.button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
