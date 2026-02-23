<script lang="ts">
	import type { PageProps } from './$types';
	let { params }: PageProps = $props();

	const post = $derived(params.slug);
</script>

<h1>{post}</h1>
<div>{@html post}</div>
