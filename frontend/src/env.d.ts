/// <reference types="vite/client" />

interface ImportMetaEnv {
    readonly GOE_WEBSOCKET_URL: string
}

interface ImportMeta {
    readonly env: ImportMetaEnv
}
