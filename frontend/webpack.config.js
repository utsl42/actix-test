'use strict';

const path = require('path');
const webpack = require('webpack');
const HtmlWebpackPlugin = require('html-webpack-plugin');

const config = {
    entry: './src/Index.bs.js',
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: '[name].[contenthash:8].js',
        libraryTarget: 'var',
        library: 'App'
    },
    plugins: [
        new webpack.HashedModuleIdsPlugin(), // so that file hashes don't change unexpectedly
        new HtmlWebpackPlugin({
            title: 'Countries',
            meta: {
                viewport: 'content=device-width, initial-scale=1',
            },
            template: 'src/index.html',
        }),
    ],
    mode: "development",

    devServer: {
        contentBase: path.join(__dirname, "dist"),
        compress: true,
        headers: {
            'Access-Control-Allow-Origin': '*',
            'Access-Control-Allow-Headers': '*',
        },
        port: 9000
    }

};

if(process.env.NODE_ENV === 'dev') {
    config.mode="development";

} else {
    config.mode="production";
    config.optimization={
        runtimeChunk: 'single',
        splitChunks: {
            chunks: 'all',
            maxInitialRequests: Infinity,
            minSize: 0,
            cacheGroups: {
                vendor: {
                    test: /[\\/]node_modules[\\/]/,
                    name(module) {
                        // get the name. E.g. node_modules/packageName/not/this/part.js
                        // or node_modules/packageName
                        const packageName = module.context.match(/[\\/]node_modules[\\/](.*?)([\\/]|$)/)[1];

                        // npm package names are URL-safe, but some servers don't like @ symbols
                        return `npm.${packageName.replace('@', '')}`;
                    },
                },
            },
        },
    };
}

module.exports = config;