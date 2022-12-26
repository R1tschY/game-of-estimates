<script lang="ts">
    export let id: string
    export let label: string
    export let value: boolean
</script>

<div class="field">
    <p class="control">
        <input type="checkbox" class="switch" {id} bind:checked={value} />

        <label for={id} class="label">{label}</label>
    </p>
</div>
