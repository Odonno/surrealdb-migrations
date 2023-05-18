import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { ViteRsw } from 'vite-plugin-rsw';

export default defineConfig({
	plugins: [sveltekit(), ViteRsw()]
});
