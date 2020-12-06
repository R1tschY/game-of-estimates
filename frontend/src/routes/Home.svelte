<script>
    import { connected, player_id, game } from '../stores.js'
    import { decks } from '../consts'
    import Banner from '../components/Banner.svelte'
    import SelectWithButton from '../components/SelectWithButton.svelte'

    let deckId = decks[0].id
    let roomId = ''

    let decks_dropdown = decks.map((deck) => {
        return {
            id: deck.id,
            label: deck.name + ' (' + deck.cards.slice(0, -2).join(', ') + ')',
        }
    })
</script>

<div>
    <Banner />
    <section class="section">
        <div class="container">
            <div class="columns is-centered">
                <form class="column is-half">
                    <div class="field has-addons">
                        <div class="control">
                            <input
                                class="input"
                                type="text"
                                placeholder="Game no."
                                bind:value={roomId} />
                        </div>
                        <div class="control">
                            <button
                                type="submit"
                                class="button is-fullwidth is-primary"
                                on:click={game.join(roomId)}>Join existing room</button>
                        </div>
                    </div>
                </form>
            </div>
            <div class="is-divider" data-content="OR" />
            <div class="columns is-centered">
                <form class="column is-half">
                    <div class="field">
                        <label class="label" for="deck_field">Deck</label>
                        <div class="control is-expanded" id="deck_field">
                            <SelectWithButton
                                items={decks_dropdown}
                                bind:value={deckId} />
                        </div>
                    </div>

                    <div class="field">
                        <div class="control">
                            <button
                                type="button"
                                class="button is-fullwidth is-warning"
                                on:click={game.create(deckId)}>Create room</button>
                        </div>
                    </div>
                </form>
            </div>
        </div>
    </section>

    <section class="section">
        <div class="container">
            <div>Connected: {$connected}</div>
            <div>Player ID: {$player_id}</div>
            <div>game ID: {$game.id}</div>
            <div>game State: {$game.status}</div>
            <div>game Error: {$game.last_error}</div>
            <div>Deck ID: {deckId}</div>
        </div>
    </section>
</div>
