<script lang="ts">
    export let value: string = ""
    export let label: string = "Copy"

    let input: HTMLInputElement

    function onClick() {
        input.select();
        document.execCommand("copy");
    }
</script>

<style lang="scss">
    .copy-link-input {
        width: 15rem;
    }
</style>

<div class="field has-addons">
    <div class="control">
        <input bind:this={input} class="input copy-link-input" type="text" readonly
            bind:value={value}>
    </div>
    <div class="control">
        <button class="button is-primary" on:click="{onClick}">{label}</button>
    </div>
</div>