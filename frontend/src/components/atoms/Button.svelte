<script lang="ts">
    interface Props {
        /** Is this the principal call to action on the page? */
        primary?: boolean
        /** How large should the button be? */
        size?: 's' | 'm' | 'l'
        /** Button contents */
        label: string
        /** The onclick event handler */
        onclick?: () => void
    }

    const { primary = false, size = 'm', label }: Props = $props()
</script>

<button type="button" class="g-btn" g-size={size} g-primary={primary}>
    {label}
</button>
