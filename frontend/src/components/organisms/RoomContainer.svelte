<script lang="ts">
    import { gameState, name, observer, players } from '$/stores'

    import { get } from 'svelte/store'
    import DisconnectedMW from '$/components/organisms/DisconnectedMW.svelte'
    import Switch from '$/components/atoms/Switch.svelte'
    import PlayerEstimate from '$/components/atoms/PlayerEstimate.svelte'
    import EstimatesControl from '$/components/molecules/EstimatesControl.svelte'
    import { client, wsService } from '$/client'
    import { m } from '$lib/paraglide/messages.js'
    import { onMount } from 'svelte'
    import RenamePlayerDialog from '$/components/organisms/RenamePlayerDialog.svelte'

    let { id } = $props()

    const open: boolean = $derived($gameState?.open ?? false)

    onMount(() => {
        console.log('onMount')
        wsService.connect()
    })

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
</script>

<section class="section">
    <div class="container box">
        <div class="columns">
            <!-- Name -->
            <div class="column is-flex is-gap-2 is-align-items-center">
                <p class="is-size-4">{$name ? $name : m.anonymous()}</p>
                <button
                    type="button"
                    class="button"
                    command="show-modal"
                    commandfor="rename-dialog">{m.rename()}</button
                >
            </div>

            <div class="column"></div>

            <!-- Voter -->
            <div class="column is-narrow">
                <Switch
                    id="player-is-voter"
                    class="player-name-control"
                    bind:value={$observer}
                    label={m.observer()}
                />
            </div>
        </div>
    </div>
</section>

<section class="section">
    <div class="container box">
        <h2 class="title is-4">{m.estimates()}</h2>
        <div class="buttons">
            <button class="button" onclick={restart}>{m.restart()}</button>
            <button class="button" disabled={open} onclick={forceOpen}
                >{m.open()}</button
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
                    {m.chooseYourEstimate()}
                </div>
            </div>
            <EstimatesControl />
        </div>
    </section>
{/if}

<DisconnectedMW />
<RenamePlayerDialog id="rename-dialog" />
