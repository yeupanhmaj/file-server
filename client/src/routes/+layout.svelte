<script lang="ts">
	import { page } from '$app/state';
	import {
		Apps,
		CloudUpload,
		Folder,
		Help,
		Search,
		Settings,
		SettingsAdjust,
		TrashCan,
		UserAvatar
	} from 'carbon-icons-svelte';
	import { resolve } from '$app/paths';

	import { Divider, UploadDialog } from '$lib';
	import { fileService } from '$lib';
	import favicon from '$lib/assets/favicon.svg';
	import { goto } from '$app/navigation';
	import type { StorageStats } from '$lib/services/types';

	let { children } = $props();
	let searchQuery = $state('');
	let showUploadDialog = $state(false);
	let storageStats = $state<StorageStats | null>(null);

	// Fetch storage stats on mount
	$effect(() => {
		const fetchStorage = async () => {
			try {
				storageStats = await fileService.getStorageStatsEndpoint();
			} catch (error) {
				console.error('Failed to fetch storage stats:', error);
			}
		};
		fetchStorage();
	});

	// Determine current path for upload
	const currentUploadPath = $derived(() => {
		// If on home page, use root
		if (page.url.pathname.startsWith('/home')) {
			return '.';
		}
		// If on folder page, use current folder path (would need to be passed from child)
		// For now, default to root

		// TODO: We should ideally have a more robust way to determine current folder path,
		// perhaps via a store or context
		return '.';
	});

	const isActive = (path: string) => {
		return page.url.pathname === path || page.url.pathname.startsWith(path + '/');
	};

	const handleSearch = (e: KeyboardEvent) => {
		if (e.key === 'Enter' && searchQuery.trim() !== '') {
			// @ts-expect-error: Query parameters are not in the type system but work at runtime
			goto(resolve(`/search?q=${encodeURIComponent(searchQuery.trim())}`));
		}
	};

	const handleUploadComplete = async () => {
		// Refresh storage stats after upload
		try {
			storageStats = await fileService.getStorageStatsEndpoint();
		} catch (error) {
			console.error('Failed to refresh storage stats:', error);
		}

		// Trigger a page reload to show new files

		// TODO: Ideally we would have a more elegant way to refresh
		// the current folder view without a full reload
		window.location.reload();
	};
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<!-- {TODO: consider create a separate layout component to reuse} -->
<div class="app">
	<!-- Header -->
	<header class="header">
		<div class="header-left">
			<div class="logo">
				<Folder size={32} />
				<span class="logo-text">File Server</span>
			</div>
		</div>

		<div class="search-bar">
			<Search size={20} class="search-icon" />
			<input
				type="text"
				placeholder="Search in files"
				bind:value={searchQuery}
				onkeydown={handleSearch}
			/>
		</div>

		<div class="header-right">
			<button class="icon-button" aria-label="Help">
				<Help size={20} />
			</button>
			<button class="icon-button" aria-label="Settings">
				<Settings size={20} />
			</button>
			<button class="icon-button" aria-label="Apps">
				<Apps size={20} />
			</button>
			<button class="icon-button avatar" aria-label="Account">
				<UserAvatar size={32} />
			</button>
		</div>
	</header>

	<!-- Main Content Area -->
	<div class="main-container">
		<!-- Sidebar -->

		<aside class="sidebar">
			<button class="new-button" onclick={() => (showUploadDialog = true)}>
				<CloudUpload size={20} />
				<span>Upload</span>
			</button>

			<nav class="nav-menu">
				<button
					onclick={() => goto(resolve('/home'))}
					class="nav-item"
					class:active={isActive('/home')}
				>
					<Folder size={20} />
					<span>My Drive</span>
				</button>
				<button
					onclick={() => goto(resolve('/trash'))}
					class="nav-item"
					class:active={isActive('/trash')}
				>
					<TrashCan size={20} />
					<span>Trash</span>
				</button>
			</nav>

			<Divider />
			<div class="storage-info">
				<SettingsAdjust size={20} />
				<div class="storage-text">
					<div class="storage-label">Storage</div>
					<div class="storage-usage">
						{#if storageStats}
							{storageStats.used_formatted} of {storageStats.total_formatted} used ({storageStats.percentage.toFixed(
								1
							)}%)
						{:else}
							Loading...
						{/if}
					</div>
				</div>
			</div>
		</aside>

		<!-- Main Content -->
		<main class="content">
			{@render children()}
		</main>
	</div>

	<!-- Upload Dialog -->
	<UploadDialog
		bind:isOpen={showUploadDialog}
		currentPath={currentUploadPath()}
		onUploadComplete={handleUploadComplete}
	/>
</div>

<style>
	:global(body) {
		margin: 0;
		padding: 0;
		font-family: 'Google Sans', 'Roboto', Arial, sans-serif;
		background-color: #fff;
	}

	.app {
		display: flex;
		flex-direction: column;
		height: 100vh;
		overflow: hidden;
	}

	/* Header Styles */
	.header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 16px;
		border-bottom: 1px solid #e0e0e0;
		background-color: #fff;
		height: 64px;
		box-sizing: border-box;
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: 16px;
		flex: 0 0 auto;
	}

	.logo {
		display: flex;
		align-items: center;
		gap: 8px;
		color: #5f6368;
	}

	.logo-text {
		font-size: 22px;
		font-weight: 400;
		color: #5f6368;
	}

	.search-bar {
		display: flex;
		align-items: center;
		background-color: #f1f3f4;
		border-radius: 8px;
		padding: 0 16px;
		flex: 1;
		max-width: 720px;
		margin: 0 auto;
		height: 48px;
	}

	.search-bar input {
		border: none;
		background: transparent;
		outline: none;
		width: 100%;
		padding: 12px 8px;
		font-size: 16px;
		color: #202124;
	}

	.search-bar input::placeholder {
		color: #5f6368;
	}

	.header-right {
		display: flex;
		align-items: center;
		gap: 4px;
		flex: 0 0 auto;
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
		transition: background-color 0.2s;
	}

	.icon-button:hover {
		background-color: #f1f3f4;
	}

	.icon-button.avatar {
		padding: 4px;
	}

	/* Main Container */
	.main-container {
		display: flex;
		flex: 1;
		overflow: hidden;
	}

	/* Sidebar Styles */
	.sidebar {
		width: 256px;
		background-color: #fff;
		padding: 16px 12px;
		border-right: 1px solid #e0e0e0;
		overflow-y: auto;
		flex-shrink: 0;
	}

	.new-button {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px 24px;
		background-color: #fff;
		border: 1px solid #dadce0;
		border-radius: 24px;
		cursor: pointer;
		font-size: 14px;
		font-weight: 500;
		margin-bottom: 16px;
		box-shadow:
			0 1px 2px 0 rgba(60, 64, 67, 0.302),
			0 1px 3px 1px rgba(60, 64, 67, 0.149);
		transition: all 0.2s;
		color: #202124;
	}

	.new-button:hover {
		background-color: #f8f9fa;
		box-shadow:
			0 1px 3px 0 rgba(60, 64, 67, 0.302),
			0 4px 8px 3px rgba(60, 64, 67, 0.149);
	}

	.nav-menu {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.nav-item {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px 12px;
		border-radius: 24px;
		border: transparent;
		color: #202124;
		text-decoration: none;
		font-size: 14px;
		transition: background-color 0.2s;
		cursor: pointer;
		background-color: transparent;
	}

	.nav-item:hover {
		background-color: #f1f3f4;
	}

	.nav-item.active {
		background-color: #e8f0fe;
		color: #1967d2;
	}

	.storage-info {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px 12px;
		color: #5f6368;
		font-size: 13px;
	}

	.storage-text {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.storage-label {
		font-weight: 500;
		color: #202124;
	}

	.storage-usage {
		font-size: 12px;
		color: #5f6368;
	}

	/* Content Area */
	.content {
		flex: 1;
		overflow: auto;
		background-color: #fff;
		padding: 24px;
	}

	/* Responsive */
	@media (max-width: 768px) {
		.search-bar {
			max-width: 300px;
		}

		.logo-text {
			display: none;
		}

		.sidebar {
			position: absolute;
			left: 0;
			top: 64px;
			height: calc(100vh - 64px);
			z-index: 100;
			box-shadow: 2px 0 8px rgba(0, 0, 0, 0.1);
		}
	}
</style>
