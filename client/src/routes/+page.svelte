<script lang="ts">
	import { goto, afterNavigate } from '$app/navigation';
	afterNavigate(() => {
		if (window.location.pathname === '/') {
			goto('/home');
		}
	});
</script>
