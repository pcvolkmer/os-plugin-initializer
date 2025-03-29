export default {
    entry: {
        main: './web/script.js',
    },
    output: {
        path: './src/resources/assets',
        chunkFilename: '[id].js'
    },
    module: {
        rules: [
            {
                test: /\.css$/,
                use: [{
                    loader: "postcss-loader",
                    options: {
                        postcssOptions: {
                            plugins: {
                                "@tailwindcss/postcss": {},
                            },
                        }
                    }
                }],
                type: "css"
            },
        ]
    },
    experiments: {
        css: true,
    }
}