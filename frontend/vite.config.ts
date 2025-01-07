import { resolve } from 'path'
import { defineConfig } from 'vite'
import { sveltekit } from '@sveltejs/kit/vite'
import { paraglide } from '@inlang/paraglide-sveltekit/vite'
import eslint from 'vite-plugin-eslint'

// https://vitejs.dev/config/
export default defineConfig({
    plugins: [
        sveltekit(),
        eslint(),
        paraglide({
            project: './project.inlang',
            outdir: './src/lib/paraglide',
        }),
    ],
    envPrefix: 'GOE_',
    css: {
        preprocessorOptions: {
            scss: {
                quietDeps: true,
            },
            sass: {
                quietDeps: true,
            },
        },
    },
})
