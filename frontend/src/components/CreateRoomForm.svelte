<script lang="ts">
    import { decks } from '../deck'
    import SelectWithButton from '../components/SelectWithButton.svelte'
    import { m } from '$lib/paraglide/messages.js'

    let deckId = decks[0].id
    let customDeck = ''

    type Action = null | 'join' | 'create'
    let action: Action = null

    let decks_dropdown = decks
        .map((deck) => {
            return {
                id: deck.id,
                label:
                    deck.name + ' (' + deck.cards.slice(0, -2).join(', ') + ')',
            }
        })
        .concat([{ id: 'custom', label: m.customDeck() }])

    function createRoom() {
        action = 'create'
    }
</script>

<form class="box p-5" method="post" action="/room">
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

    <div class="field">
        <div class="control">
            <div
                class="is-flex is-flex-direction-row is-justify-content-center"
            >
                <button
                    type="submit"
                    class="button is-primary"
                    class:is-loading={action === 'create'}
                    on:click={createRoom}>{m.createRoom()}</button
                >
            </div>
        </div>
    </div>
</form>
