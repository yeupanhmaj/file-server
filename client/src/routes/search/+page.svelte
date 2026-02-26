<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { folderService } from '$lib';
	import { Document, Folder } from 'carbon-icons-svelte';
	import type { FileSystemItem } from '$lib/services/types';

	// Get the search query from URL parameter
	const searchQuery = $derived(page.url.searchParams.get('q') || '');

	// Track accumulated results and pagination state
	let currentPage = $state(1);
	let allResults = $state<FileSystemItem[]>([]);
	let searchMetadata = $state<{ total: number; hasMore: boolean } | null>(null);
	let isLoadingMore = $state(false);
	let lastQuery = $state('');

	// Perform search when query changes
	const searchFiles = async (pageNum: number, append: boolean = false) => {
		if (!searchQuery.trim()) {
			return null;
		}

		// Reset if it's a new search query
		if (searchQuery !== lastQuery) {
			currentPage = 1;
			allResults = [];
			lastQuery = searchQuery;
			append = false;
		}

		try {
			const response = await folderService.searchFiles({
				search_string: searchQuery,
				path: '.',
				page: pageNum,
				limit: 5
			});

			if (append) {
				allResults = [...allResults, ...response.results];
			} else {
				allResults = response.results;
			}

			searchMetadata = {
				total: response.total,
				hasMore: response.has_more
			};

			return response;
		} catch (error) {
			console.error('Search error:', error);
			return null;
		}
	};

	// Initial search effect
	$effect(() => {
		if (searchQuery) {
			searchFiles(1, false);
		} else {
			allResults = [];
			searchMetadata = null;
			lastQuery = '';
		}
	});

	const loadMore = async () => {
		if (isLoadingMore || !searchMetadata?.hasMore) return;

		isLoadingMore = true;
		currentPage += 1;
		await searchFiles(currentPage, true);
		isLoadingMore = false;
	};

	const handleItemClick = async (file: FileSystemItem) => {
		if (file.item_type === 'folder') {
			await goto(resolve(`/folder/${file.id}`));
		}
		// For files, we could implement a preview or download action
	};
</script>

<div class="search-page">
	<div class="search-header">
		<h1>Search Results</h1>
		{#if searchQuery}
			<p class="search-query">Searching for: <strong>"{searchQuery}"</strong></p>
		{/if}
	</div>

	<div class="results-container">
		{#if !searchQuery.trim()}
			<div class="empty-state">
				<p>Enter a search term to find files</p>
			</div>
		{:else if allResults.length === 0 && searchMetadata}
			<div class="empty-state">
				<p>No results found for "{searchQuery}"</p>
			</div>
		{:else if allResults.length > 0}
			<div class="results-list">
				<div class="results-count">
					Found {searchMetadata?.total || 0}+ {searchMetadata?.total === 1 ? 'item' : 'items'}
					{#if searchMetadata?.hasMore}
						<span class="more-indicator">(showing {allResults.length})</span>
					{/if}
				</div>
				{#each allResults as file (file.id)}
					<button class="result-item" onclick={() => handleItemClick(file)} type="button">
						<div class="result-icon">
							{#if file.item_type === 'folder'}
								<Folder size={24} />
							{:else}
								<Document size={24} />
							{/if}
						</div>
						<div class="result-info">
							<div class="result-name">{file.name}</div>
							<div class="result-path">{file.path}</div>
						</div>
						<div class="result-meta">
							<div class="result-size">{file.size}</div>
							<div class="result-modified">{file.modified}</div>
						</div>
					</button>
				{/each}

				{#if searchMetadata?.hasMore}
					<div class="load-more-container">
						<button class="load-more-btn" onclick={loadMore} disabled={isLoadingMore} type="button">
							{isLoadingMore ? 'Loading...' : 'Load More'}
						</button>
					</div>
				{/if}
			</div>
		{/if}
	</div>
</div>

<style>
	.search-page {
		max-width: 1200px;
		margin: 0 auto;
	}

	.search-header {
		margin-bottom: 24px;
	}

	.search-header h1 {
		font-size: 28px;
		font-weight: 400;
		color: #202124;
		margin: 0 0 8px 0;
	}

	.search-query {
		color: #5f6368;
		font-size: 14px;
		margin: 0;
	}

	.search-query strong {
		color: #202124;
	}

	.results-container {
		background: white;
		border-radius: 8px;
	}

	.empty-state {
		padding: 48px;
		text-align: center;
		color: #5f6368;
	}

	.results-list {
		display: flex;
		flex-direction: column;
		gap: 1px;
	}

	.results-count {
		padding: 12px 16px;
		font-size: 14px;
		color: #5f6368;
		border-bottom: 2px solid #f1f3f4;
		font-weight: 500;
	}

	.more-indicator {
		color: #1a73e8;
		font-size: 13px;
		margin-left: 8px;
	}

	.result-item {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 12px 16px;
		border: none;
		border-bottom: 1px solid #f1f3f4;
		background: white;
		cursor: pointer;
		transition: background-color 0.2s;
		width: 100%;
		text-align: left;
	}

	.result-item:hover {
		background-color: #f8f9fa;
	}

	.result-item:last-child {
		border-bottom: none;
	}

	.result-icon {
		color: #5f6368;
		display: flex;
		align-items: center;
		flex-shrink: 0;
	}

	.result-info {
		flex: 1;
		min-width: 0;
	}

	.result-name {
		font-size: 14px;
		color: #202124;
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.result-path {
		font-size: 12px;
		color: #5f6368;
		margin-top: 4px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.result-meta {
		display: flex;
		gap: 24px;
		flex-shrink: 0;
	}

	.result-size,
	.result-modified {
		font-size: 12px;
		color: #5f6368;
	}

	.result-size {
		min-width: 80px;
		text-align: right;
	}

	.result-modified {
		min-width: 140px;
		text-align: right;
	}

	.load-more-container {
		padding: 16px;
		display: flex;
		justify-content: center;
		border-top: 1px solid #f1f3f4;
	}

	.load-more-btn {
		padding: 10px 24px;
		background: #1a73e8;
		color: white;
		border: none;
		border-radius: 4px;
		font-size: 14px;
		font-weight: 500;
		cursor: pointer;
		transition: background-color 0.2s;
	}

	.load-more-btn:hover:not(:disabled) {
		background: #1765cc;
	}

	.load-more-btn:disabled {
		background: #dadce0;
		cursor: not-allowed;
	}
</style>
