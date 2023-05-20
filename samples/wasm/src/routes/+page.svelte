<script lang="ts">
	import { onMount } from 'svelte';
	import init, { setup, get_blog_posts } from 'rsw-hello';

	type BlogPost = {
		id: string;
		title: string;
		content: string;
		status: string;
	};

	let appReady = false;

	onMount(async () => {
		console.log('onMount');

		await init();
		await setup();

		appReady = true;
	});

	const getBlogPosts = async () => {
		const posts = await get_blog_posts();
		return posts as BlogPost[];
	};
</script>

<h1>Welcome to My Blog!</h1>

{#if appReady}
	{#await getBlogPosts()}
		<p>Loading...</p>
	{:then posts}
		{#each posts as post (post.id)}
			<article>
				<h2>{post.title}</h2>
				<p>{post.content}</p>

				<footer>
					<span>{post.status}</span>
				</footer>
			</article>
		{:else}
			<p>No blog post!</p>
		{/each}
	{:catch error}
		<p style="color: red">{error.message}</p>
	{/await}
{:else}
	<p>Loading...</p>
{/if}
