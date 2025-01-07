// @ts-check

import globals from 'globals'
import prettier from 'eslint-config-prettier'
import eslint from '@eslint/js'
import tseslint from 'typescript-eslint'
import eslintPluginSvelte from 'eslint-plugin-svelte'
import prettierConfig from 'eslint-config-prettier'
import svelteConfig from './svelte.config.js'

export default tseslint.config(
    eslint.configs.recommended,
    ...tseslint.configs.strict,
    ...tseslint.configs.stylistic,
    ...eslintPluginSvelte.configs['flat/recommended'],
    prettier,
    ...eslintPluginSvelte.configs['flat/prettier'],
    prettierConfig,
    {
        files: ['**/*.svelte'],
        languageOptions: {
            parserOptions: {
                parser: tseslint.parser,
                extraFileExtensions: ['.svelte'],
                svelteConfig,
            },
        },
        rules: {
            'svelte/no-at-html-tags': 'warn',
        },
    },
    {
        languageOptions: {
            ecmaVersion: 12,
            sourceType: 'module',
            globals: {
                ...globals.browser,
            },
            parserOptions: {
                projectService: true,
                tsconfigRootDir: import.meta.dirname,
                extraFileExtensions: ['.svelte'],
            },
        },
        rules: {
            '@typescript-eslint/no-explicit-any': 'warn',
            '@typescript-eslint/no-non-null-assertion': 'off',
        },
    },
)
