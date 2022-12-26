<script lang="ts">
    interface SelectItem {
        id: string
        label: string
    }

    export let items: SelectItem[]
    export let value: string | undefined = undefined
</script>

<div class="select is-fullwidth">
    <select bind:value>
        {#each items as item (item.id)}
            <option value={item.id}>{item.label}</option>
        {/each}
    </select>
</div>
