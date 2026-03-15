<script lang="ts">
    import Dialog from '$/components/atoms/Dialog.svelte'
    import { name } from '$/stores'
    import { m } from '$lib/paraglide/messages.js'

    let { id } = $props()

    let newName = $state($name)
    let open = $state(false)
    let errorMessage: string | undefined = $state()

    function onSubmit(event: SubmitEvent) {
        event.preventDefault()
        $name = newName
        open = false
    }
</script>

<Dialog closedby="any" {id} {open}>
    <form onsubmit={onSubmit}>
        <div class="field">
            <label class="label" for="name">{m.newName()}</label>
            <div class="control">
                <input
                    type="text"
                    class="input is-expanded"
                    id="name"
                    name="name"
                    bind:value={newName}
                />
            </div>
        </div>

        <div class="field is-grouped-centered">
            <div class="control">
                <button type="submit" class="button is-primary">
                    {m.rename()}
                </button>
            </div>
            <p class="help is-danger">{errorMessage}</p>
        </div>
    </form>
</Dialog>
