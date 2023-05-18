<script lang="ts">
	import { onMount } from 'svelte';
	import init, { greet, setup } from 'rsw-hello';

	onMount(async () => {
		console.log('onMount');

		await init();
		await setup();
	});
</script>

<h1>Welcome to SvelteKit</h1>
<p>Visit <a href="https://kit.svelte.dev">kit.svelte.dev</a> to read the documentation</p>

<button on:click={greet}>Greet</button>
