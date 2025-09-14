import { resolve } from 'path'
import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import eslint from 'vite-plugin-eslint'

// https://vitejs.dev/config/
export default defineConfig({
    plugins: [svelte(), eslint()],
    envPrefix: 'GOE_',
    build: {
        rollupOptions: {
            input: {
                createRoom: resolve(__dirname, 'src/entries/create-room.ts'),
                createRoomDemo: resolve(__dirname, 'create-room.html'),
                room: resolve(__dirname, 'src/entries/room.ts'),
                icons: resolve(__dirname, 'src/entries/icons.ts'),
                style: resolve(__dirname, 'src/main.sass'),
            },
            output: {
                assetFileNames: 'assets/[name][extname]',
                chunkFileNames: 'assets/[name].js',
                entryFileNames: 'assets/[name].js',
            },
        },
    },
})
