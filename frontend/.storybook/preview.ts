import type { Preview } from '@storybook/svelte-vite'

import '../src/styles/goe/main.scss'

const preview: Preview = {
    parameters: {
        controls: {
            matchers: {
                color: /(background|color)$/i,
                date: /Date$/i,
            },
        },
    },
}

export default preview
