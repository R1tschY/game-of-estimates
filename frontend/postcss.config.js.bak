/* eslint-disable @typescript-eslint/no-var-requires */
import purgecss from '@fullhuman/postcss-purgecss'

export default {
    plugins: [
        purgecss({
            content: ['./src/**/*.{svelte,js,ts}'], // declaring source files
            safelist: {
                standard: [/^svelte-/], // required for inline component styles
            },
            variables: true, // remove unused CSS variables
        }),
    ],
}
