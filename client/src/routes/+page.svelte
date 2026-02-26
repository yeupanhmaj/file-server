<script lang="ts">
	import { resolve } from '$app/paths';
	import { goto, afterNavigate } from '$app/navigation';

	afterNavigate(() => {
		if (window.location.pathname === '/') {
			goto(resolve('/home'));
		}
	});
</script>
