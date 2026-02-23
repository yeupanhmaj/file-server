<script lang="ts">
	import { Folder, Document, OverflowMenuVertical, Grid, List } from 'carbon-icons-svelte';
	import TreeView from '$lib/components/TreeView.svelte';

	let viewMode = $state<'grid' | 'list'>('list');

	// Sample hierarchical data for tree view
	const treeData = [
		{
			id: 1,
			name: 'Documents',
			type: 'folder' as const,
			children: [
				{
					id: 11,
					name: 'Work',
					type: 'folder' as const,
					children: [
						{ id: 111, name: 'Reports', type: 'folder' as const, children: [] },
						{ id: 112, name: 'Budget 2024.xlsx', type: 'file' as const }
					]
				},
				{
					id: 12,
					name: 'Personal',
					type: 'folder' as const,
					children: [
						{ id: 121, name: 'Resume.pdf', type: 'file' as const },
						{ id: 122, name: 'Cover Letter.docx', type: 'file' as const }
					]
				}
			]
		},
		{
			id: 2,
			name: 'Images',
			type: 'folder' as const,
			children: [
				{ id: 21, name: 'Vacation 2024', type: 'folder' as const, children: [] },
				{ id: 22, name: 'Screenshots', type: 'folder' as const, children: [] }
			]
		},
		{
			id: 3,
			name: 'Projects',
			type: 'folder' as const,
			children: [
				{
					id: 31,
					name: 'Website',
					type: 'folder' as const,
					children: [
						{ id: 311, name: 'src', type: 'folder' as const, children: [] },
						{ id: 312, name: 'public', type: 'folder' as const, children: [] }
					]
				}
			]
		}
	];

	// Sample flat data for list/grid views
	const files = [
		{ id: 1, name: 'Documents', type: 'folder', modified: '2024-02-20', size: '-' },
		{ id: 2, name: 'Images', type: 'folder', modified: '2024-02-19', size: '-' },
		{ id: 3, name: 'Project Report.pdf', type: 'file', modified: '2024-02-18', size: '2.4 MB' },
		{
			id: 4,
			name: 'Meeting Notes.txt',
			type: 'file',
			modified: '2024-02-15',
			size: '12 KB'
		},
		{ id: 5, name: 'Presentation.pptx', type: 'file', modified: '2024-02-10', size: '5.8 MB' }
	];

	let selectedNodeId = $state<string | number | null>(null);

	const toggleView = () => {
		if (viewMode === 'list') {
			viewMode = 'grid';
		} else {
			viewMode = 'list';
		}
	};

	const handleNodeClick = (node: any) => {
		selectedNodeId = node.id;
		console.log('Clicked node:', node);
	};
</script>

<div class="container">
	<div class="toolbar">
		<h2 class="page-title">My Drive</h2>
		<div class="toolbar-right">
			<button
				class="view-toggle"
				onclick={toggleView}
				aria-label="Toggle view"
				title="View: {viewMode}"
			>
				{#if viewMode === 'list'}
					<Grid size={20} />
				{:else}
					<List size={20} />
				{/if}
			</button>
		</div>
	</div>

	{#if viewMode === 'list'}
		<div class="file-list">
			<div class="file-list-header">
				<div class="file-name">Name</div>
				<div class="file-modified">Last modified</div>
				<div class="file-size">File size</div>
				<div class="file-actions"></div>
			</div>

			{#each files as file}
				<div class="file-row">
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
	{:else}
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
	{/if}
</div>

<style>
	.container {
		max-width: 1400px;
		margin: 0 auto;
	}

	.toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 24px;
		padding-bottom: 16px;
		border-bottom: 1px solid #e0e0e0;
	}

	.page-title {
		font-size: 24px;
		font-weight: 400;
		color: #202124;
		margin: 0;
	}

	.toolbar-right {
		display: flex;
		gap: 8px;
	}

	.view-toggle {
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
	/* Tree View Styles */
	.tree-container {
		background-color: #fff;
		border: 1px solid #e0e0e0;
		border-radius: 8px;
		padding: 8px;
		max-width: 600px;
	}

	/* 

	.view-toggle:hover {
		background-color: #f1f3f4;
	}

	/* List View Styles */
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

	/* Grid View Styles */
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

	/* Responsive */
	@media (max-width: 768px) {
		.file-list-header,
		.file-row {
			grid-template-columns: 1fr 48px;
		}

		.file-modified,
		.file-size {
			display: none;
		}

		.file-grid {
			grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
		}
	}
</style>
