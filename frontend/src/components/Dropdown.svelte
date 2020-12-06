<script>
    export let items;
    export let active;

    let open = false;

    $: label = getActiveLabel(active)

    function getActiveLabel(_active) {
        let activeItem = items.find(i => i.id == _active)
        return activeItem ? activeItem.label : ""
	}

    function handleOpen() {
		open = !open;
    }
    
    function clickItem(item) {
		active = item.id
	}
</script>


<div class="dropdown" on:click={handleOpen} class:is-active="{open}">
    <div class="dropdown-trigger">
        <button class="button is-fullwidth" aria-haspopup="true" aria-controls="dropdown-menu">
            <span>{label}</span>
            <span class="icon is-small">
                <i class="fas fa-angle-down" aria-hidden="true"></i>
            </span>
        </button>
    </div>
    <div class="dropdown-menu" id="dropdown-menu" role="menu">
        <div class="dropdown-content">
            {#each items as item (item.id)}
                <a href="{item.href}" class="dropdown-item" class:is-active="{item.id === active}" on:click={clickItem(item)}>
                    {item.label}
                </a>
            {/each}
        </div>
    </div>
</div>