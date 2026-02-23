<script lang="ts">
	import { Folder, FolderOpen, ChevronRight, ChevronDown } from 'carbon-icons-svelte';
	import TreeView from './TreeView.svelte';

	interface TreeNode {
		id: string | number;
		name: string;
		type: 'folder' | 'file';
		children?: TreeNode[];
		metadata?: any;
	}

	interface Props {
		nodes: TreeNode[];
		level?: number;
		onNodeClick?: (node: TreeNode) => void;
		selectedId?: string | number | null;
	}

	let { nodes, level = 0, onNodeClick, selectedId = null }: Props = $props();

	// Track which nodes are expanded (open)
	let expandedNodes = $state<Set<string | number>>(new Set());

	const toggleNode = (nodeId: string | number) => {
		const newExpanded = new Set(expandedNodes);
		if (newExpanded.has(nodeId)) {
			newExpanded.delete(nodeId);
		} else {
			newExpanded.add(nodeId);
		}
		expandedNodes = newExpanded;
	};

	const handleNodeClick = (node: TreeNode) => {
		if (node.type === 'folder') {
			toggleNode(node.id);
		}
		onNodeClick?.(node);
	};

	const isExpanded = (nodeId: string | number) => expandedNodes.has(nodeId);
	const hasChildren = (node: TreeNode) => node.children && node.children.length > 0;
</script>

<div class="tree-view">
	{#each nodes as node}
		<div class="tree-node" style="--level: {level}">
			<button
				class="node-content"
				class:selected={selectedId === node.id}
				onclick={() => handleNodeClick(node)}
			>
				<div class="node-icon-wrapper">
					{#if node.type === 'folder' && hasChildren(node)}
						{#if isExpanded(node.id)}
							<ChevronDown size={16} class="chevron" />
						{:else}
							<ChevronRight size={16} class="chevron" />
						{/if}
					{:else if node.type === 'folder'}
						<span class="chevron-placeholder"></span>
					{/if}

					{#if node.type === 'folder'}
						{#if isExpanded(node.id)}
							<FolderOpen size={20} />
						{:else}
							<Folder size={20} />
						{/if}
					{/if}
				</div>

				<span class="node-label">{node.name}</span>
			</button>

			{#if node.type === 'folder' && hasChildren(node) && isExpanded(node.id)}
			<TreeView nodes={node.children || []} level={level + 1} {onNodeClick} {selectedId} />
			{/if}
		</div>
	{/each}
</div>

<style>
	.tree-view {
		user-select: none;
	}

	.tree-node {
		display: flex;
		flex-direction: column;
	}

	.node-content {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 8px;
		padding-left: calc(8px + var(--level) * 20px);
		background: none;
		border: none;
		cursor: pointer;
		color: #202124;
		font-size: 14px;
		border-radius: 4px;
		transition: background-color 0.2s;
		text-align: left;
		width: 100%;
		box-sizing: border-box;
	}

	.node-content:hover {
		background-color: #f1f3f4;
	}

	.node-content.selected {
		background-color: #e8f0fe;
		color: #1967d2;
	}

	.node-icon-wrapper {
		display: flex;
		align-items: center;
		gap: 4px;
		color: #5f6368;
		flex-shrink: 0;
	}

	.node-content.selected .node-icon-wrapper {
		color: #1967d2;
	}

	.chevron-placeholder {
		width: 16px;
		height: 16px;
		display: inline-block;
	}

	.node-label {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		flex: 1;
	}

	:global(.chevron) {
		transition: transform 0.2s;
	}
</style>
