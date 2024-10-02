export default [
    {
        root: true,
        parserOptions: {
            ecmaVersion: 12,
            sourceType: 'module',
        },
        env: {
            es6: true,
            browser: true,
        },
        extends: [
            'eslint:recommended',
            'prettier',
            'plugin:@typescript-eslint/recommended',
            'plugin:svelte/prettier',
        ],
        parser: '@typescript-eslint/parser',
        plugins: [],
        overrides: [
            {
                files: ['*.svelte'],
                parser: 'svelte-eslint-parser',
                parserOptions: {
                    parser: '@typescript-eslint/parser',
                },
            },
        ],
        rules: {
            '@typescript-eslint/no-explicit-any': 'warn',
        },
        settings: {
            'svelte3/typescript': true,
        },
    },
]
