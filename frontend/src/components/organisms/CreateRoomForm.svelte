<script lang="ts">
    import { decks } from '$/deck'
    import SelectWithButton from '$/components/atoms/SelectWithButton.svelte'
    import { m } from '$lib/paraglide/messages.js'
    import { client } from '$/client'

    let deckId = $state(decks[0].id)
    let customDeck = $state('')

    let errorMessage = $state()

    let decks_dropdown = decks
        .map((deck) => {
            return {
                id: deck.id,
                label:
                    deck.name + ' (' + deck.cards.slice(0, -2).join(', ') + ')',
            }
        })
        .concat([{ id: 'custom', label: m.customDeck() }])

    function submit(evt: SubmitEvent) {
        evt.preventDefault()

        errorMessage = null

        client
            .createRoom(deckId, customDeck)
            .then((location) => {
                window.location.href = location
            })
            .catch((error) => {
                console.error("Couldn't create room:", error)
                errorMessage = "Couldn't create room"
            })
    }
</script>

<form class="box p-5" onsubmit={submit}>
    <div class="field">
        <label class="label" for="deck_field">{m.deck()}</label>
        <div class="control is-expanded">
            <SelectWithButton
                id="deck_field"
                items={decks_dropdown}
                name="deck"
                bind:value={deckId}
            />
        </div>
    </div>
    {#if deckId === 'custom'}
        <div class="field">
            <label class="label" for="custom_deck_field"
                >{m.customDeckField()}</label
            >
            <div class="control">
                <input
                    type="text"
                    placeholder={m.customDeckPlaceholder()}
                    class="input is-expanded"
                    id="custom_deck_field"
                    name="custom_deck"
                    bind:value={customDeck}
                />
            </div>
            <p class="help">{m.customDeckHelp()}</p>
        </div>
    {/if}

    <div class="field is-grouped-centered">
        <div class="control">
            <button type="submit" class="button is-primary">
                {m.createRoom()}
            </button>
        </div>
        <p class="help is-danger">{errorMessage}</p>
    </div>
</form>
