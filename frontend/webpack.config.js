const MiniCssExtractPlugin = require('mini-css-extract-plugin')
const path = require('path')
const webpack = require('webpack')
const svelte_preprocess = require('svelte-preprocess')

const mode = process.env.NODE_ENV || 'development'
const prod = mode === 'production'

require('dotenv').config({
    path: path.resolve('..', '.env.' + mode),
})

module.exports = {
    entry: {
        bundle: ['./src/main.ts'],
    },
    resolve: {
        alias: {
            svelte: path.resolve('node_modules', 'svelte'),
        },
        extensions: ['.mjs', '.ts', '.svelte', '.js'],
        mainFields: ['svelte', 'browser', 'module', 'main'],
        symlinks: false,
    },
    output: {
        path: __dirname + '/public',
        filename: '[name].js',
        chunkFilename: '[name].[id].js',
    },
    devServer: {
        historyApiFallback: true,
    },
    module: {
        rules: [
            {
                test: /\.tsx?$/,
                use: 'ts-loader',
                exclude: /node_modules/,
            },
            {
                test: /\.(html|svelte)$/,
                use: {
                    loader: 'svelte-loader',
                    options: {
                        compilerOptions: {
                            dev: !prod,
                        },
                        emitCss: prod,
                        hotReload: !prod,
                        dev: !prod,
                        hotOptions: {
                            preserveLocalState: false,
                            noPreserveStateKey: '@!hmr',
                            noReload: false,
                            optimistic: false,
                        },
                        preprocess: svelte_preprocess([
                            svelte_preprocess.scss(),
                            svelte_preprocess.typescript(),
                        ]),
                    },
                },
            },
            {
                test: /node_modules\/svelte\/.*\.mjs$/,
                resolve: {
                    fullySpecified: false,
                },
            },
            {
                test: /\.css$/,
                use: [
                    prod ? MiniCssExtractPlugin.loader : 'style-loader',
                    'css-loader',
                    'postcss-loader',
                ],
            },
            {
                test: /\.s[ac]ss$/i,
                use: [
                    prod ? MiniCssExtractPlugin.loader : 'style-loader',
                    'css-loader',
                    'sass-loader',
                ],
            },
        ],
    },
    mode,
    plugins: [
        new MiniCssExtractPlugin({
            filename: '[name].css',
        }),
        new webpack.EnvironmentPlugin({
            NODE_ENV: mode,
            GOE_WEBSOCKET_URL: '',
        }),
    ],
    devtool: prod ? false : 'source-map',
}
