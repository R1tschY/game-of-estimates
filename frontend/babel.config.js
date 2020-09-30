module.exports = {
	presets: [
		['@babel/preset-env', {
			modules: 'commonjs',
		}],
	],
	plugins: [
		'add-module-exports',
		"@babel/plugin-transform-strict-mode"
	],
};