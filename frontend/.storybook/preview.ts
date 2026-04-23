import type { Preview } from '@storybook/sveltekit'

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
