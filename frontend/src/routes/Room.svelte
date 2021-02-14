<style lang="scss">
    $card-width: 63mm * 0.25;
    $card-height: 88mm * 0.25;

    $tt-gray: #63666a;
    $tt-orange: #ed8b00;
    $tt-petrol: #208CA3;

    .card-inner {
        border: 1px solid #eee;
        border-radius: 7px 0 7px 0;
        margin: 3px;
        line-height: calc(#{$card-height} - 10px);
        vertical-align: middle;
    }

    .game-card {
        width: $card-width;
        height: $card-height;
        padding: 0;
        font-size: 1.7em;
        text-align: center;
        display: inline-block;
        // line-height: 3em;
        margin: 0rem 0.25rem 0.5rem 0.25rem;
        border: 1px solid $tt-gray;
        box-shadow: 0 0em 0.5em -0.125em rgba(10, 10, 10, 0.1);
        outline: 0;
        transition: transform 1s, margin-top 0.2s;
        border-radius: 10px 0 10px 0;
        transform: rotateY(0deg);
    }

    .game-card.selectable {
        cursor: pointer;
    }  

    .game-card.selected {
        background-color: #{adjust-color($tt-orange, $lightness: 40%)};
        //color: white;
        margin-top: -0.5rem;
    }

    .game-card-item {
        display: inline-block;
        vertical-align: top;
        white-space: nowrap;
        list-style: none;
    }

    .game-card-back {
        background-color: $tt-orange;
        color: black;
        padding: 2px;
        backface-visibility: hidden;
        transform: rotateY(180deg);
        position: absolute;
    }

    .game-card-front {
        background-color: white;
        padding: 2px;
        backface-visibility: hidden;
        transform: rotateY(0deg);
        position: absolute;
    }

    .game-card-normal {
        position: unset;
        background-color: white;
    }

    .backcover > .game-card-back {
        transform: rotateY(0deg);
    }

    .backcover > .game-card-front {
        transform: rotateY(180deg);
    }

    .hidden > .game-card {
        display: none;
    }

    .empty {
        border: 2px solid transparent;
        box-shadow: inset 0 0em 0.5em -0.125em rgba(10, 10, 10, 0.1);
        z-index: -10;
        position: relative;
    }

    .card-row {
        height: calc(#{$card-height} + 4rem);
        margin: 10px 0;
    }
</style>

<script lang="ts">
    import { playerId, vote, voter, debug, players, gameState, name as nameStore } from '../stores'
    import Banner from '../components/Banner.svelte'
    import CopyLink from '../components/CopyLink.svelte'
    import { get_deck as getDeck } from '../deck'
    import { client, playerState } from '../client'
    import { get } from 'svelte/store';
    import DisconnectedMW from '../components/DisconnectedMW.svelte'
    import SingleTextInput from '../components/SingleTextInput.svelte'
    import Switch from '../components/Switch.svelte'

    export let id: string | null = null

    let name: string = get(nameStore)

    $: cards = $gameState ? getDeck($gameState.deck).cards : []
    $: open = $gameState && $gameState.open


    // TODO: disconnect on unmount
    client.welcome.connect(() => {
        const state = get(client.state)
        if (id !== null && state !== "joining" && state !== "joined") {
            console.log('init join', id)
            client.joinRoom(id)
        }
    })

    function setVote(value: string | null) {
        vote.update((v) => {
            return v !== value ? value : null
        })
    }

    function forceOpen() {
        if (!open) {
            client.forceOpen()
        }
    }

    function restart() {
        client.restart()
    }

    function changeName(evt: CustomEvent) {
        nameStore.set(name ? name : null)
    }
</script>

<div>
    <Banner />

    <section class="section">
        <div class="container">
            <div class="columns">
                <!-- Voter -->
                <div class="column is-narrow">
                    <Switch id="player-is-voter" bind:value={$voter} label="Voter" />
                </div>

                <div class="column"></div>

                <!-- Name -->
                <div class="column">
                    <SingleTextInput 
                        id="player-name"
                        action="Change name"
                        placeholder="Player name"
                        bind:value={name}
                        on:submit={changeName} />
                </div>

                <div class="column"></div>

                <!-- Link -->
                <div class="column is-narrow">
                    <CopyLink value={document.location + ""} label="Copy room link" />
                </div>
            </div>
        </div>
    </section>

    <section class="section">
        <div class="container">
            <h2 class="title is-4">Estimates</h2>
            <div class="buttons">
                <button
                    class="button is-primary is-light"
                    on:click={restart}>Restart</button>
                <button
                    class="button is-primary is-light"
                    on:click={forceOpen}>Open</button>
            </div>
            <ul class="card-row">
                {#each $players as player (player.id)}
                {#if player.voter}
                <li class="game-card-item">
                    <div>
                        <div class:backcover={!open} class:hidden={!player.vote}>
                            <div class="game-card game-card-back">
                                <div class="card-inner">♠️</div>
                            </div>
                            <div class="game-card game-card-front">
                                <div class="card-inner">{player.vote ? player.vote : '\xA0'}</div>
                            </div>
                        </div>
                        <div class="game-card empty"></div>
                    </div>
                    <div style="text-align:center;">
                        {player.name}
                    </div>
                </li>
                {/if}
                {/each}
            </ul>
        </div>
    </section>

    {#if $voter}
        <section class="section">
            <div class="container">
                <h2 class="title is-4">Choose your estimate</h2>
                <ul class="card-row">
                    {#each cards as card}
                        <li class="game-card-item">
                            <button
                                class="game-card game-card-normal selectable"
                                on:click={() => setVote(card)}
                                class:selected={$vote === card}>
                                <div class="card-inner">{card}</div>
                            </button>
                        </li>
                    {/each}
                </ul>
            </div>
        </section>
    {/if}

    {#if $debug}
        <section class="section">
            <div class="container">
                <div>player state: {$playerState}</div>
                <div>Player ID: {$playerId}</div>
                <div>ID: {id}</div>
                <div>vote: {$vote}</div>
                <div>voter: {$voter}</div>
                <div>Open: {open}</div>
                <div>players: {JSON.stringify($players)}</div>
            </div>
        </section>
    {/if}

    <DisconnectedMW />
</div>
