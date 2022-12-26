<script lang="ts">
    import { createEventDispatcher } from 'svelte'

    export let id: string
    export let value: string
    export let action: string
    export let placeholder: string | null = null

    const dispatch = createEventDispatcher()

    function onSubmit() {
        dispatch('submit', value)
    }
</script>

<div class="field has-addons">
    <div class="control">
        <input
            class="input"
            type="text"
            {id}
            {placeholder}
            bind:value
            on:keypress={(evt) =>
                evt.key === 'Enter' ? onSubmit() : undefined}
        />
    </div>
    <div class="control">
        <button class="button is-primary" on:click={onSubmit}>
            {action}
        </button>
    </div>
</div>
