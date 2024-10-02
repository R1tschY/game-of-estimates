<script lang="ts">
    import { decks } from '../deck'
    import SelectWithButton from '../components/SelectWithButton.svelte'
    import { getText } from '../i18n'

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
        .concat([{ id: 'custom', label: getText('customDeck') }])

    function createRoom() {
        action = 'create'
    }
</script>

<div class="field">
    <label class="label" for="deck_field">{getText('deck')}</label>
    <div class="control is-expanded" id="deck_field">
        <SelectWithButton
            items={decks_dropdown}
            name="deck"
            bind:value={deckId}
        />
    </div>
</div>
{#if deckId === 'custom'}
    <div class="field">
        <label class="label" for="deck_field"
            >{getText('customDeckField')}</label
        >
        <div class="control">
            <input
                type="text"
                placeholder={getText('customDeckPlaceholder')}
                class="input is-expanded"
                id="custom_deck_field"
                name="custom_deck"
                bind:value={customDeck}
            />
        </div>
        <p class="help">{getText('customDeckHelp')}</p>
    </div>
{/if}

<div class="field">
    <div class="control">
        <div class="is-flex is-flex-direction-row is-justify-content-center">
            <button
                type="submit"
                class="button is-primary"
                class:is-loading={action === 'create'}
                on:click={createRoom}>{getText('createRoom')}</button
            >
        </div>
    </div>
</div>
