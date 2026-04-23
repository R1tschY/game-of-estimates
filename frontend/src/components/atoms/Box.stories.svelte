<script module>
    import { defineMeta } from '@storybook/addon-svelte-csf'
    import Box from '$/components/atoms/Box.svelte'

    // More on how to set up stories at: https://storybook.js.org/docs/writing-stories
    const { Story } = defineMeta({
        title: 'Atoms/Box',
        component: Box,
        tags: ['autodocs'],
    })
</script>

<!-- More on writing stories with args: https://storybook.js.org/docs/writing-stories/args -->
<Story name="Default" />
