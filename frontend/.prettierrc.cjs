module.exports = {
    singleQuote: true,
    semi: false,
    tabWidth: 4,
    trailingComma: 'all',
    plugins: ['prettier-plugin-svelte'],
    overrides: [{ files: "*.svelte", options: { parser: "svelte" } }]
}
