<script lang="ts">
    let {
        children,
        open = false,
        id = undefined,
        closedby = undefined,
    } = $props()

    let dialog: HTMLDialogElement | undefined = $state()

    $effect(() => {
        if (dialog) {
            if (open) {
                if (!dialog.open) {
                    dialog.showModal()
                }
            } else {
                if (dialog.open) {
                    dialog.close()
                }
            }
        }
    })

    function handleDialogClick(evt: Event) {
        if (evt.target === dialog) {
            open = false
        }
    }
</script>

<dialog
    bind:this={dialog}
    {closedby}
    {id}
    onclose={() => (open = false)}
    onclick={handleDialogClick}
>
    {@render children?.()}
</dialog>
