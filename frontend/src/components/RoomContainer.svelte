<script lang="ts">
    import { gameState, name as nameStore, players, observer } from '../stores'

    import { client } from '../client'
    import { get } from 'svelte/store'
    import DisconnectedMW from '../components/DisconnectedMW.svelte'
    import SingleTextInput from '../components/SingleTextInput.svelte'
    import Switch from '../components/Switch.svelte'
    import PlayerEstimate from '../components/PlayerEstimate.svelte'
    import EstimatesControl from '../components/EstimatesControl.svelte'
    import { getText } from '../i18n'

    export let id: string

    let name: string = get(nameStore) ?? ''

    let open: boolean
    $: open = $gameState?.open ?? false

    // TODO: disconnect on unmount
    client.welcome.connect(() => {
        const state = get(client.state)
        if (state !== 'joining' && state !== 'joined') {
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

    function changeName() {
        nameStore.set(name ? name : null)
    }
</script>

<section class="section">
    <div class="container box">
        <div class="columns">
            <!-- Name -->
            <div class="column player-name-control">
                <SingleTextInput
                    id="player-name"
                    action="✓"
                    placeholder={getText('playerNamePlaceholder')}
                    bind:value={name}
                    on:submit={changeName}
                />
            </div>

            <div class="column"></div>

            <!-- Voter -->
            <div class="column is-narrow">
                <Switch
                    id="player-is-voter"
                    class="player-name-control"
                    bind:value={$observer}
                    label={getText('observer')}
                />
            </div>
        </div>
    </div>
</section>

<section class="section">
    <div class="container box">
        <h2 class="title is-4">{getText('estimates')}</h2>
        <div class="buttons">
            <button class="button is-primary is-light" on:click={restart}
                >{getText('restart')}</button
            >
            <button
                class="button is-primary is-light"
                disabled={open}
                on:click={forceOpen}>{getText('open')}</button
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

{#if !$observer}
    <section class="section">
        <div class="container">
            <div class="columns is-centered">
                <div class="column is-narrow">
                    {getText('chooseYourEstimate')}
                </div>
            </div>
            <EstimatesControl />
        </div>
    </section>
{/if}

<DisconnectedMW />
