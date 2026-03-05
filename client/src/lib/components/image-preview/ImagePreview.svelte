<script lang="ts">
	import { Close, Download } from 'carbon-icons-svelte';

	let {
		imageUrl = '',
		imageName = '',
		isOpen = $bindable(false),
		onDownload = () => {}
	} = $props();

	const handleClose = () => {
		isOpen = false;
	};

	const handleBackdropClick = (e: MouseEvent) => {
		if (e.target === e.currentTarget) {
			handleClose();
		}
	};

	const handleDownloadClick = () => {
		onDownload();
		handleClose();
	};

	// Close on Escape key
	const handleKeydown = (e: KeyboardEvent) => {
		if (e.key === 'Escape' && isOpen) {
			handleClose();
		}
	};
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
	<div class="modal-backdrop" onclick={handleBackdropClick} role="presentation">
		<div class="modal-content">
			<div class="modal-header">
				<h2 class="modal-title">{imageName}</h2>
				<div class="modal-actions">
					<button
						class="icon-button"
						onclick={handleDownloadClick}
						aria-label="Download image"
						title="Download"
					>
						<Download size={24} />
					</button>
					<button
						class="icon-button close-button"
						onclick={handleClose}
						aria-label="Close preview"
						title="Close"
					>
						<Close size={24} />
					</button>
				</div>
			</div>
			<div class="modal-body">
				<img src={imageUrl} alt={imageName} class="preview-image" />
			</div>
		</div>
	</div>
{/if}

<style>
	.modal-backdrop {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background-color: rgba(0, 0, 0, 0.85);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 9999;
		padding: 20px;
		animation: fadeIn 0.2s ease-out;
	}

	@keyframes fadeIn {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}

	.modal-content {
		background: #ffffff;
		border-radius: 12px;
		max-width: 90vw;
		max-height: 90vh;
		display: flex;
		flex-direction: column;
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
		animation: slideUp 0.3s ease-out;
	}

	@keyframes slideUp {
		from {
			transform: translateY(20px);
			opacity: 0;
		}
		to {
			transform: translateY(0);
			opacity: 1;
		}
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
		border-bottom: 1px solid #e0e0e0;
	}

	.modal-title {
		font-size: 18px;
		font-weight: 500;
		color: #202124;
		margin: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		flex: 1;
		margin-right: 16px;
	}

	.modal-actions {
		display: flex;
		gap: 8px;
	}

	.icon-button {
		background: none;
		border: none;
		cursor: pointer;
		padding: 8px;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #5f6368;
		transition: all 0.2s;
	}

	.icon-button:hover {
		background-color: #f1f3f4;
		color: #202124;
	}

	.close-button:hover {
		background-color: #fce8e6;
		color: #d93025;
	}

	.modal-body {
		padding: 20px;
		overflow: auto;
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 1;
		min-height: 0;
	}

	.preview-image {
		max-width: 100%;
		max-height: calc(90vh - 100px);
		object-fit: contain;
		border-radius: 4px;
	}

	@media (max-width: 768px) {
		.modal-content {
			max-width: 95vw;
			max-height: 95vh;
		}

		.modal-header {
			padding: 12px 16px;
		}

		.modal-title {
			font-size: 16px;
		}

		.modal-body {
			padding: 12px;
		}

		.preview-image {
			max-height: calc(95vh - 80px);
		}
	}
</style>
