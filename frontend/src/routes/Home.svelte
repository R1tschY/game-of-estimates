<script lang="ts">
  import {connected, connecting, debug, lastError, playerId,} from '../stores'
  import {decks} from '../deck'
  import {client} from '../client'
  import Header from '../components/Header.svelte'
  import Footer from '../components/Footer.svelte'
  import SelectWithButton from '../components/SelectWithButton.svelte'
  import DisconnectedMW from '../components/DisconnectedMW.svelte'
  import {getText} from '../i18n'

  let deckId = decks[0].id
    let roomId = ''

    type Action = null | 'join' | 'create'
    let action: Action = null

    let decks_dropdown = decks.map((deck) => {
        return {
            id: deck.id,
            label: deck.name + ' (' + deck.cards.slice(0, -2).join(', ') + ')',
        }
    })

    function createRoom() {
        action = 'create'
        client.createRoom(deckId)
    }

    function joinRoom() {
        action = 'join'
        client.joinRoom(roomId)
    }

    // TODO: disconnect
    client.state.subscribe((state) => {
        if (state !== 'joining') {
            action = null
        }
    })
</script>

<div>
    <Header />

    <section class="section">
        <div class="container">
            <h1 class="title is-4">{getText('summary')}</h1>
            <p>{@html getText('description')}</p>
        </div>
    </section>

    {#if $lastError}
        <section class="section">
            <div class="container">
                <div class="notification is-danger">
                    <button class="delete" />
                    {$lastError}
                </div>
            </div>
        </section>
    {/if}

    <section class="section">
        <div class="container">
            <div class="columns is-centered">
                <form class="create-room-form column is-half">
                    <div class="field">
                        <label class="label" for="deck_field"
                            >{getText('deck')}</label
                        >
                        <div class="control is-expanded" id="deck_field">
                            <SelectWithButton
                                items={decks_dropdown}
                                bind:value={deckId}
                            />
                        </div>
                    </div>

                    <div class="field">
                        <div class="control">
                            <div class="columns is-centered">
                                <div class="column is-narrow">
                                    <button
                                        type="button"
                                        class="button is-primary"
                                        class:is-loading={action === 'create'}
                                        on:click={createRoom}
                                        >{getText('createRoom')}</button
                                    >
                                </div>
                            </div>
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
