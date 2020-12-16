const MiniCssExtractPlugin = require('mini-css-extract-plugin')
const path = require('path')
const webpack = require('webpack')

const mode = process.env.NODE_ENV || 'development'
const prod = mode === 'production'

require('dotenv').config({ 
    path: path.resolve('..', '.env.' + mode),
})

const { scss } = require('svelte-preprocess')

module.exports = {
    entry: {
        bundle: ['./src/main.js'],
    },
    resolve: {
        alias: {
            svelte: path.resolve('node_modules', 'svelte'),
        },
        extensions: ['.mjs', '.js', '.svelte'],
        mainFields: ['svelte', 'browser', 'module', 'main'],
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
                test: /\.svelte$/,
                use: {
                    loader: 'svelte-loader',
                    options: {
                        emitCss: true,
                        hotReload: !prod,
                        preprocess: require('svelte-preprocess')([scss()]),
                    },
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
            'NODE_ENV': 'development',
            'GOE_WEBSOCKET_URL': 'ws://localhost:5500'
        })
    ],
    devtool: prod ? false : 'source-map',
}
