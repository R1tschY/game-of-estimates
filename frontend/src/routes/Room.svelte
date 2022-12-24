<script lang="ts">
    import {debug, gameState, name as nameStore, playerId, players, vote, voter,} from '../stores'
    import Header from '../components/Header.svelte'
    import CopyLink from '../components/CopyLink.svelte'

    import {client, playerState} from '../client'
    import {get} from 'svelte/store'
    import DisconnectedMW from '../components/DisconnectedMW.svelte'
    import SingleTextInput from '../components/SingleTextInput.svelte'
    import Switch from '../components/Switch.svelte'
    import Footer from '../components/Footer.svelte'
    import PlayerEstimate from '../components/PlayerEstimate.svelte'
    import EstimatesControl from '../components/EstimatesControl.svelte'
    import {getText} from "../i18n";

    export let id: string | null = null

    let name: string = get(nameStore)

    $: open = $gameState?.open ?? false

    // TODO: disconnect on unmount
    client.welcome.connect(() => {
        const state = get(client.state)
        if (id !== null && state !== 'joining' && state !== 'joined') {
            console.log('init join', id)
            client.joinRoom(id)
        }
    })

    function forceOpen() {
        if (!open) {
            client.forceOpen()
        }
    }

    function restart() {
        client.restart()
    }

    function changeName(_evt: CustomEvent) {
        nameStore.set(name ? name : null)
    }
</script>

<div>
    <Header />

    <section class="section">
        <div class="container">
            <div class="columns">
                <!-- Voter -->
                <div class="column is-narrow">
                    <Switch
                        id="player-is-voter"
                        bind:value={$voter}
                        label={getText("voter")}
                    />
                </div>

                <div class="column"></div>

                <!-- Name -->
                <div class="column">
                    <SingleTextInput
                        id="player-name"
                        action={getText("changeName")}
                        placeholder="Player name"
                        bind:value={name}
                        on:submit={changeName}
                    />
                </div>

                <div class="column"></div>

                <!-- Link -->
                <div class="column is-narrow">
                    <CopyLink
                        value={document.location + ''}
                        label={getText("copyRoomLink")}
                    />
                </div>
            </div>
        </div>
    </section>

    <section class="section">
        <div class="container">
            <h2 class="title is-4">{getText("estimates")}</h2>
            <div class="buttons">
                <button class="button is-primary is-light" on:click={restart}
                    >{getText("restart")}</button
                >
                <button
                    class="button is-primary is-light"
                    disabled={open}
                    on:click={forceOpen}>{getText("open")}</button
                >
            </div>
            <ul class="game-board">
                {#each $players as player (player.id)}
                    {#if player.voter}
                        <PlayerEstimate {player} {open} />
                    {/if}
                {/each}
            </ul>
        </div>
    </section>

    {#if $voter}
        <section class="section">
            <div class="container">
                <div class="columns is-centered">
                    <div class="column is-narrow">{getText("chooseYourEstimate")}</div>
                </div>
                <EstimatesControl />
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
    <Footer />
</div>
