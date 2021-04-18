<script lang="ts">
    import { connected, connecting, playerId, lastError, state, debug } from '../stores'
    import { decks } from '../deck'
    import { client } from '../client'
    import Header from '../components/Header.svelte'
    import Footer from '../components/Footer.svelte'
    import SelectWithButton from '../components/SelectWithButton.svelte'
    import DisconnectedMW from '../components/DisconnectedMW.svelte'
    import NProgress from "nprogress"

    let deckId = decks[0].id
    let roomId = ''

    type Action = null | "join" | "create"
    let action: Action = null

    let decks_dropdown = decks.map((deck) => {
        return {
            id: deck.id,
            label: deck.name + ' (' + deck.cards.slice(0, -2).join(', ') + ')',
        }
    })

    function createRoom() {
        action = "create"
        NProgress.start()
        client.createRoom(deckId)
    }

    function joinRoom() {
        action = "join"
        NProgress.start()
        client.joinRoom(roomId)
    }

    // TODO: disconnect
    client.state.subscribe((state) => {
        if (state !== "joining") {
            action = null
            NProgress.done()
        }
    })
</script>

<div>
    <Header />

    {#if $lastError}
    <section class="section">
        <div class="container">
            <div class="notification is-danger">
                <button class="delete"></button>
                {$lastError}
            </div>
        </div>
    </section>
    {/if}

    <section class="section">
        <div class="container">
            <div class="columns is-centered">
                <form class="column is-half">
                    <div class="field has-addons">
                        <div class="control">
                            <input
                                class="input"
                                type="text"
                                placeholder="Room no."
                                bind:value={roomId} />
                        </div>
                        <div class="control">
                            <button
                                type="button"
                                class="button is-fullwidth is-primary"
                                class:is-loading={action === "join"}
                                on:click={joinRoom}>Join existing room</button>
                        </div>
                    </div>
                </form>
            </div>
            <div class="divider">OR</div>
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
                                class:is-loading={action === "create"}
                                on:click={createRoom}>Create room</button>
                        </div>
                    </div>
                </form>
            </div>
        </div>
    </section>

    {#if $debug}
        <section class="section">
            <div class="container">
                <div>Connected: {$connected}</div>
                <div>Connecting: {$connecting}</div>
                <div>Player ID: {$playerId}</div>
                <div>Deck ID: {deckId}</div>
            </div>
        </section>
    {/if}

    <DisconnectedMW />
    <Footer />
</div>
