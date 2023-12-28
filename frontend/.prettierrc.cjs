module.exports = {
    singleQuote: true,
    semi: false,
    tabWidth: 4,
    trailingComma: 'all',
    pluginSearchDirs: ['src'],
    plugins: ['prettier-plugin-svelte'],
    overrides: [{ files: "*.svelte", options: { parser: "svelte" } }]
}
