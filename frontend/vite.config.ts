import { paraglideVitePlugin } from '@inlang/paraglide-js'
import { defineConfig } from 'vite'
import { sveltekit } from '@sveltejs/kit/vite'
import eslint from 'vite-plugin-eslint'

// https://vitejs.dev/config/
export default defineConfig({
    plugins: [
        sveltekit(),
        eslint(),
        paraglideVitePlugin({
            project: './project.inlang',
            outdir: './src/lib/paraglide',
            strategy: ['url', 'cookie', 'baseLocale'],
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
    resolve: {
        alias: {
            $: 'src',
        },
    },
})
