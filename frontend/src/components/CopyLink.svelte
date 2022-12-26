<script lang="ts">
    export let value = ''
    export let label = 'Copy'

    let input: HTMLInputElement

    function onClick() {
        input.select()
        document.execCommand('copy')
    }
</script>

<div class="field has-addons">
    <div class="control">
        <input
            bind:this={input}
            class="input copy-link-input"
            type="text"
            readonly
            bind:value
        />
    </div>
    <div class="control">
        <button class="button is-primary" on:click={onClick}>{label}</button>
    </div>
</div>

<style lang="scss">
    .copy-link-input {
        width: 15rem;
    }
</style>
