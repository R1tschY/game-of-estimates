<script lang="ts">
    export let items;
    export let value = "";
</script>

<div class="select is-fullwidth">
    <select bind:value={value}>
        {#each items as item (item.id)}
            <option value={item.id}>{item.label}</option>
        {/each}
    </select>
</div>